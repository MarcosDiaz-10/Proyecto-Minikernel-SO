# Proyecto-Minikernel-SO

Este readme es provisional, solo para ustedes ya que tengo otras materias y no quiero ponerlo más bonito la verdad

Como instalar el proyecto desde el .tar que les pase: 
me da flojera subirlo a docker hub JAJJAJAJJAJAJJAAJAJ
```
docker load -i mi-proyecto.tar
docker run  -v "$(pwd)/archivos_para_procesar:/input" mi-proyecto-rust
```

Si alguien se atreve a querer compilarlo seria: 

```
Cargo run
```

Más nada Rust es maravillo


Consideraciones hasta ahora:

- La pila esta en el proceso de usuario va a tener un espacio de 50 direcciones y va a construirse de forma descendente.
- El store solo funciona con direccionamiento directo e indexado ya que estas son como tal direcciones de memoria.
- El resto de instrucciones tipo saltos o load, en el direccionamiento directo/indexado las direcciones funcionan como punteros.
- El verctor de interrupciones se va a cargar en memoria del proyecto, donde cuando se genera una interrupción se colaca la direccion de la instruccion que va a contener el opcode de la interrupción simulando lo real.
- permitir que se hagan saltos indirecto en j, es decir, cuando el modo de direccionamiento sea distinto a inmediato. Lo que va a suceder es que la dirección se comporta como un puntero
- En el procesador se va a tener un objeto temporal del dma y este se comunica a traves de canales con el dma real enviandole ese objeto temporal
- Se va a guardar un el vector de insterrupciones en memoria, con los opcode desde 90 al 98 van a referirse a la insterrupciones y se van a cargar en los primeros 8 espacios de memoria
- El input lo recibe en la carpeta  /input en la raiz
- Para cargar programas es `load nombre_archivo dir_mem`
- Para correr programas es `run modo nombre_programa`
- Para seguir en modo debugger se usa el comando `next`
- Para terminar el proyecto en general se usa `exit`
- Considero que la cuenta de las lineas empieza desde 1
- Los programas del input se suben en `/input` de la raiz (Por si lo compilan), en docker ahí les pase el comando y allí ponen los programas a cargar

Cosas que faltan:

- Los logs
- Interrupciones de reloj
- Mejorar la terminación del programa
- Probar todas las funcionalidades