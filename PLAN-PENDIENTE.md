# Plan pendiente (cambios de plan del usuario, 2026-10-26)

PARTE A: CAMBIOS EN LA FASE 3 (matemáticas)
1. Objetivo principal nuevo: reconocer mi letra y SUSTITUIRLA por trazos limpios y legibles, como la conversión de tinta a formas. Cada símbolo reconocido se redibuja limpio en la misma posición y tamaño que el mío, sin reordenar nada. Deben ser trazos normales de Rnote, seleccionables y borrables, sin cambiar el formato .rnote (si hace falta, avísame antes). Si un símbolo es dudoso, se deja mi trazo original. Debe poder deshacerse, activarse y desactivarse en ajustes, y tener un umbral de confianza configurable.
2. Se mantiene lo ya diseñado: grupos por forma, contexto, glosario, comprobación dimensional y corrección por símbolo o por grupo. Tras una corrección, el símbolo limpio se actualiza.
3. El resultado del "=" en gris pasa a ser una función aparte, activable y desactivable en ajustes. Define qué hará con ecuaciones a despejar (2x+1=7, x=), simplificar y derivar.
4. Fuente de los símbolos limpios: trazos de una sola línea o contornos de una fuente libre de matemáticas. Elige una y apunta su licencia.

PARTE B: FUNCIONES EXTRA DE ONENOTE (sustituye la regla "no añadir funciones que no estén en la lista")
Compruébalas en el código y apúntalas en AUDITORIA.md como existe, parcial o no existe, con la ruta del archivo. Solo implementa lo que falte, una rama por función, con tests donde se pueda y documentada en CAMBIOS-ONEPEPE.md con cómo probarla yo con el lápiz.

FASE 1b: botones del lápiz Wacom (prioridad alta)
- Botón 1: borrador de trazo entero mientras está pulsado; al soltar vuelve a la herramienta anterior.
- Botón 2: selección con lazo mientras está pulsado.
- Ambos reasignables desde ajustes a cualquier herramienta, en modo "mientras pulso" o "alterna al pulsar".
- Si el lápiz tiene extremo borrador, debe funcionar. El README de Rnote (Pitfalls) dice que no maneja bien los eventos "Eraser Tool": resuélvelo o explica por qué no se puede y qué alternativa hay.
- Borrador de trazo entero en la barra de herramientas, además del borrador por área en tamaños pequeño, mediano y grande.
- Hecho cuando está configurable en ajustes y CAMBIOS-ONEPEPE.md explica cómo verificar con evtest qué evento envía cada botón.

FASE 1c: texto, símbolos y utilidades (por orden de prioridad)
1. Subíndice y superíndice en el texto escrito a teclado.
2. Catálogo de símbolos para insertar con un clic: alfabeto griego, operadores, ±, ∂, ∇, ∫, ∑, √, ≈, ≠, ≤, ≥, →.
3. Resaltador con paleta personalizable, semitransparente, que no tape la tinta.
4. Ejes cartesianos 2D y 3D como forma predefinida.
5. Botón para estampar fecha, hora o ambas.
6. Negrita, cursiva, subrayado, tachado y color de fuente en el texto, si faltan.
7. Casillas de verificación, viñetas y listas numeradas.
Solo si sobra tiempo y crédito: estilos rápidos (Título 1, Título 2, cita, código) y recorte de pantalla.

NO HACER: copiar formato, adjuntar archivos como icono, búsqueda de imágenes en línea, tinta con texturas, editor de ecuaciones nativo.

ORDEN SI HAY QUE RECORTAR POR CRÉDITO O TIEMPO (el crédito caduca el 5 de noviembre a las 8:59 CET):
1) hojas infinitas comprobadas, 2) tinta a formas, 3) matemáticas, 4) botones del lápiz (1b), 5) resto de la Fase 1, 6) Fase 1c, 7) cuadernos y secciones.
