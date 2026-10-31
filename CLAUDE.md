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
  en `VERSIONES.md` qué cambia. Ahora: 1.3.2.
- Distingue compilado / tests / ejecutado en la nube / probado con lápiz real. **No des por probado lo que solo
  compila.** Apunta las decisiones en `DECISIONES.md` y el estado en `ESTADO.md`.
