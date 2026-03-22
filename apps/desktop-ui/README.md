# Desktop UI (Fase 1)

Frontend estatico para conversar com o daemon e descobrir/baixar modelos remotos, com shell Tauri para desktop.

## Fluxo

1. Inicie o daemon em `http://127.0.0.1:11435`.
2. Rode o shell Tauri em `src-tauri` (recomendado, app nativo).
3. Opcional: abra `ui/index.html` para validar a UI sem Tauri.

## Ajustar endpoint

Na UI, o endpoint padrao e `http://127.0.0.1:11435`.
Use o campo `Daemon URL` para alterar e salvar.

## Recursos da UI

- Aba `Chat`: selecao de modelo local e conversa com o provider MLX.
- Aba `Descobrir Modelos`: busca no catalogo remoto (Hugging Face), cards de modelos e download em 1 clique.
- Painel `Downloads`: status em tempo real (queued/running/completed/failed) e destino local.
