Instrucciones
-------------

- Descargar el dataset de https://www.kaggle.com/datasets/najzeko/steam-reviews-2021 
- Guardar y descomprimir en un path conocido.
- Implementar el código según el enunciado https://concurrentes-fiuba.github.io/2025_1C_tp1.html

Ejecución
---------

```
cargo run <input-path> <num-threads> <output-file-name>
```

por ejemplo

```
cargo run ~/Downloads/dataset 4 output.json
```

Pruebas
-------

- La salida de la ejecución con el dataset completo debe ser igual a la del archivo `expected_output.json`, sin importar
  el orden de aparición de las keys en los mapas.