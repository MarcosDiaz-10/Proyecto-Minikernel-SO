#![allow(warnings)]
mod hardware;
mod utils;

use std::{
    io, path, process,
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use hardware::{
    architecture::Palabra, disk::Disk, interrupts::Interrups, ram::Ram, registers::Registros,
};

use crate::{
    hardware::{
        cpu::{
            Cpu, Registers_Cpu_Config, Result_Execute, Result_Execute_program, Result_Instruction,
        },
        dma::{Dma, Dma_Config, State_Dma},
        instructions::{self, Instruction},
        interrupts::{External_interrupt, handle_interrupt},
        ram,
    },
    utils::{convert_to_string_format_pal, linear_search_program, load_program_in_ram},
};

enum Mode_Execute {
    normal,
    debbuger,
    off,
}

#[derive(Debug)]
pub struct Programs {
    pub name: String,
    pub num_instruccions_with_pila: i32,
    pub pos_start_mem: i32,
    pub pos_start_program: i32,
}

impl Programs {
    pub fn new() -> Self {
        Programs {
            name: "".to_string(),
            num_instruccions_with_pila: -1,
            pos_start_mem: -1,
            pos_start_program: -1,
        }
    }
}

fn main() {
    let mut ram = Arc::new(Mutex::new(Ram::new()));
    let mut external_interrupts = Arc::new(Mutex::new(External_interrupt::new()));
    let (tx_dma, rx_dma) = mpsc::channel::<Dma_Config>();
    let (tx_terminal, rx_terminal) = mpsc::channel::<Result_Execute>();
    let (tx_cpu, rx_cpu) = mpsc::channel::<Registers_Cpu_Config>();
    let mut handles = vec![];
    let mut table_proccess: Vec<Programs> = vec![];

    {
        let mut men = ram.lock().unwrap();
        for i in (0..9) {
            let code_interrupt = Palabra::new(&format!("9{i}000000").to_string()).unwrap();
            men.writeMemory(i, code_interrupt);
        }
    }

    let mut cpu = Cpu::new(Arc::clone(&ram), Arc::clone(&external_interrupts), tx_dma);

    let cpu_thread = thread::spawn(move || {
        loop {
            cpu.result_last_program.result_program = Result_Execute_program::Succes;
            match rx_cpu.recv() {
                Ok(cpu_config) => match cpu_config.mode {
                    Mode_Execute::debbuger => match cpu.have_user_program {
                        true => {
                            cpu.result_last_program.dir_inst = cpu.registers.psw.pc;
                            cpu.step();
                            cpu.result_last_program.instruction =
                                cpu.registers.ir.conver_to_palabra();

                            tx_terminal.send(cpu.result_last_program.clone());
                            sleep(Duration::from_millis(500));

                            continue;
                        }
                        false => {
                            if cpu_config.pc == -1 {
                                cpu.result_last_program.result_program =
                                    Result_Execute_program::Error;

                                cpu.result_last_program.result_instruction =
                                    Result_Instruction::String(String::from(
                                        "No hay programa en ejecución",
                                    ));
                                tx_terminal.send(cpu.result_last_program.clone());
                                continue;
                            }

                            cpu.registers.set_rb(cpu_config.rb);
                            cpu.registers.set_rl(cpu_config.rl);
                            cpu.registers.set_rx(cpu_config.rx);
                            cpu.registers.set_sp(cpu_config.sp);
                            cpu.registers.psw.set_pc(cpu_config.pc);
                            cpu.have_user_program = true;
                            cpu.result_last_program.dir_inst = cpu.registers.psw.pc;
                            cpu.step();
                            cpu.result_last_program.instruction =
                                cpu.registers.ir.conver_to_palabra();

                            tx_terminal.send(cpu.result_last_program.clone());
                            sleep(Duration::from_millis(500));

                            continue;
                        }
                    },
                    Mode_Execute::normal => {
                        cpu.registers.set_rb(cpu_config.rb);
                        cpu.registers.set_rl(cpu_config.rl);
                        cpu.registers.set_rx(cpu_config.rx);
                        cpu.registers.set_sp(cpu_config.sp);
                        cpu.registers.psw.set_pc(cpu_config.pc);
                        cpu.run();

                        tx_terminal.send(cpu.result_last_program.clone());
                        continue;
                    }
                    Mode_Execute::off => {
                        println!("--- APAGANDO CPU ---");
                        cpu.dma_temp.state = State_Dma::Off;
                        cpu.sender_dma.send(cpu.dma_temp);
                        break;
                    }
                },
                Err(_) => {
                    println!("Error al recibir el mensaje cpu");
                    break;
                }
            }
        }
    });
    handles.push(cpu_thread);

    let mem_dma = Arc::clone(&ram);
    let external_interrupt_dma = Arc::clone(&external_interrupts);

    let dma_thread = thread::spawn(move || {
        let mut dma = Dma::new();
        let mut disk = Disk::new();
        loop {
            match rx_dma.recv() {
                Ok(dma_config) => {
                    if dma_config.state == State_Dma::Off {
                        println!("--- APAGANDO DMA ---");
                        break;
                    }

                    dma.pista_acceder = dma_config.pista_acceder;
                    dma.cil_acceder = dma_config.cil_acceder;
                    dma.sector_acceder = dma_config.sector_acceder;
                    dma.pos_men = dma_config.pos_men;
                    dma.modo = dma_config.modo;

                    dma.execute(&mut disk, &mem_dma, &external_interrupt_dma);
                }
                Err(_) => {
                    println!("Error al recibir orden dma")
                }
            }
        }
    });
    handles.push(dma_thread);

    let exit = false;
    while !exit {
        //Variables necesarias para recibir el comando
        let mut current_inst = String::new();
        let mut params_inst = String::new();
        let mut buffer = String::new();

        //Lectura del comando
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");

        //Limpieza del comando
        let command = buffer.trim().to_lowercase();

        //Desarmado del comando
        for (i, sp) in command.split_whitespace().enumerate() {
            match i {
                //Se toma el comando
                0 => {
                    current_inst = sp.to_string();
                }
                //Se guardan los argumentos del comando
                1 => {
                    params_inst = sp.to_string();
                }
                _ => {
                    params_inst += &(" ".to_string() + sp);
                }
            }
        }

        //match para determinar cual es el comando
        match current_inst.as_str() {
            "load" => {
                //variables para cargar el archivo
                let mut name_arch = String::new();
                let mut dir = -1;

                //Ciclo para dividir los parametros
                for (i, sp) in params_inst.split_whitespace().enumerate() {
                    match i {
                        0 => {
                            //Nombre archivo
                            name_arch = sp.to_string();
                        }
                        //direccion a guardar
                        1 => dir = sp.parse::<i32>().unwrap(),
                        _ => (),
                    }
                }

                if dir == -1 || name_arch == "" {
                    println!("-> Error en los parametros de carga");
                    continue;
                }

                //ruta del programa a cargar
                let path = &format!("input/{}.txt", name_arch);
                //Funcion para cargar archivo
                let res = load_program_in_ram(path, &mut table_proccess, Arc::clone(&ram), dir);
                //Respuesta de la función
                match res {
                    Ok(()) => println!("-> Programa cargado correctamente"),
                    Err(e) => {
                        println!("Error al cargar el programa: {:?}", e);
                        continue;
                    }
                }

                println!("-> Tabla de procesos: {:?}", table_proccess);
            }
            "run" => {
                //Variable de los parametros que recibe
                let mut mode = String::new();
                let mut name_prog = String::new();

                //Separando los parametros
                for (i, sp) in params_inst.split_whitespace().enumerate() {
                    match i {
                        0 => {
                            //Nombre archivo
                            mode = sp.to_string();
                        }
                        //direccion a guardar
                        1 => name_prog = sp.to_string(),
                        _ => (),
                    }
                }

                match mode.as_str() {
                    "normal" => {
                        let result_search = linear_search_program(&table_proccess, &name_prog);

                        match result_search {
                            Ok(program) => {
                                tx_cpu.send(Registers_Cpu_Config {
                                    mode: Mode_Execute::normal,
                                    rb: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem,
                                    ))
                                    .unwrap(),
                                    rl: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem + program.num_instruccions_with_pila,
                                    ))
                                    .unwrap(),
                                    rx: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem
                                            + program.num_instruccions_with_pila / 2,
                                    ))
                                    .unwrap(),
                                    sp: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem + program.num_instruccions_with_pila,
                                    ))
                                    .unwrap(),
                                    pc: program.pos_start_mem + (program.pos_start_program - 1),
                                });
                            }
                            Err(E) => {
                                println!("->Error al buscar el programa: {:?}", E);
                                continue;
                            }
                        }

                        let result_execute = rx_terminal.recv();
                        match result_execute {
                            Ok(re) => match re.result_program {
                                Result_Execute_program::Succes => {
                                    println!("-> El programa termino correctamente");
                                }
                                Result_Execute_program::Error => {
                                    println!("-> Result instrucción : {:?}", re.result_instruction);
                                    println!("-> El programa termino incorrectamente");
                                }
                            },
                            Err(e) => println!("Error al esperar al cpu: {e}"),
                        }
                    }
                    "debugger" => {
                        let result_search = linear_search_program(&table_proccess, &name_prog);

                        match result_search {
                            Ok(program) => {
                                tx_cpu.send(Registers_Cpu_Config {
                                    mode: Mode_Execute::debbuger,
                                    rb: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem,
                                    ))
                                    .unwrap(),
                                    rl: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem + program.num_instruccions_with_pila,
                                    ))
                                    .unwrap(),
                                    rx: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem
                                            + (program.num_instruccions_with_pila / 2),
                                    ))
                                    .unwrap(),
                                    sp: Palabra::new(&convert_to_string_format_pal(
                                        program.pos_start_mem + program.num_instruccions_with_pila,
                                    ))
                                    .unwrap(),
                                    pc: program.pos_start_mem + (program.pos_start_program - 1),
                                });
                            }
                            Err(E) => {
                                println!("->Error al buscar el programa: {:?}", E);
                                continue;
                            }
                        }
                        let result_execute = rx_terminal.recv();
                        match result_execute {
                            Ok(re) => match re.result_program {
                                Result_Execute_program::Succes => {
                                    println!("-> El programa ejecuto la instrucción correctamente");
                                    println!("-> Dir instrucción : {}", re.dir_inst);
                                    println!("-> Instrucción : {:?}", re.instruction);
                                    println!("-> Result instrucción : {:?}", re.result_instruction);
                                }
                                Result_Execute_program::Error => {
                                    println!("-> El programa termino incorrectamente");
                                }
                            },
                            Err(e) => println!("Error al esperar al cpu: {e}"),
                        }
                    }
                    _ => {
                        println!("Modo de ejecucion invalildo");
                        continue;
                    }
                }
            }
            "next" => {
                tx_cpu.send(Registers_Cpu_Config {
                    mode: Mode_Execute::debbuger,
                    rb: Palabra::new("00000000").unwrap(),
                    rl: Palabra::new("00000000").unwrap(),
                    rx: Palabra::new("00000000").unwrap(),
                    sp: Palabra::new("00000000").unwrap(),
                    pc: -1,
                });

                let result_execute = rx_terminal.recv();
                match result_execute {
                    Ok(re) => match re.result_program {
                        Result_Execute_program::Succes => {
                            println!("-> El programa ejecuto la instrucción correctamente");
                            println!("-> Dir instrucción : {}", re.dir_inst);
                            println!("-> Instrucción : {:?}", re.instruction);
                            println!("-> Result instrucción : {:?}", re.result_instruction);
                        }
                        Result_Execute_program::Error => {
                            println!("-> Result instrucción : {:?}", re.result_instruction);
                            println!("-> El programa termino incorrectamente");
                        }
                    },
                    Err(e) => println!("Error al esperar al cpu: {e}"),
                }
            }

            "exit" => {
                println!("--- APAGANDO SISTEMA ---");
                tx_cpu
                    .send(Registers_Cpu_Config {
                        mode: Mode_Execute::off,
                        rb: Palabra::new(&"00000000").unwrap(),
                        rl: Palabra::new(&"00000000").unwrap(),
                        rx: Palabra::new(&"00000000").unwrap(),
                        sp: Palabra::new(&"00000000").unwrap(),
                        pc: 0,
                    })
                    .unwrap();
                break;
            }
            _ => {
                println!("Comando no reconocido");
            }
        }
    }

    for handle in handles {
        handle.join();
    }
}
