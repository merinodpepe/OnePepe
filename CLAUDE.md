# Instrucciones para Claude (OnePepe)

**Lee primero [`CONTEXTO-IA.md`](CONTEXTO-IA.md)**: tiene todo el contexto del proyecto (qué es, reglas, arquitectura,
cada función, formatos, cómo compilar y probar, estado y trampas conocidas).

Reglas que no se rompen nunca (detalle en `CONTEXTO-IA.md`, sección 3):

- Responde al usuario **en español**, simple y con comandos copiables.
- `main` es OnePepe (el Rnote original está en `rnote-original`, que no se toca). Trabaja en `feat/<nombre>`,
  fusiona en `onepepe` con `--no-ff` cuando compile y pasen los tests y avanza `main` con `git push origin onepepe:main`
  (avance rápido). **Nunca** `push --force` ni borrar ramas. Solo el repositorio `merinodpepe/OnePepe`.
- **No cambies el formato `.rnote`** sin avisar antes al usuario y explicar la alternativa (archivo oculto, índice
  oculto, ajustes, `#[serde(skip)]`).
- **Nada de modelos, datasets ni datos personales en git.** Mantén la licencia GPL y sus cabeceras.
- **Versiones:** cada tanda con algo nuevo sube la versión (`meson.build` y `app.metainfo.xml.in.in`) y se apunta
  en `VERSIONES.md` qué cambia. Ahora: 1.9.0.
- Distingue compilado / tests / ejecutado en la nube / probado con lápiz real. **No des por probado lo que solo
  compila.** Apunta las decisiones en `DECISIONES.md` y el estado en `ESTADO.md`.
- **Leer una vez, no en bucle.** Antes de cambiar nada: lista todos los archivos que necesitas, léelos todos de una
  vez en paralelo, haz todas las ediciones y ejecuta los tests. Prohibido leer archivo por archivo o releer trozos
  contiguos en bucle. Si eres un subagente y los tests fallan, no iteres: escala al orquestador
  ([`ARQUITECTURA-EJECUCION.md`](docs/planes/ARQUITECTURA-EJECUCION.md) §4).

Para ejecutar el cierre de matemáticas (tareas T0–T13): [`ARQUITECTURA-EJECUCION.md`](docs/planes/ARQUITECTURA-EJECUCION.md).
Índice de toda la documentación: [`docs/README.md`](docs/README.md).
