# Starter JSON-RPC independente

Python >=3.11, biblioteca padrão, Apache-2.0. Memória simulada, sem protocolo
HID/R1, dependências Qt ou imagens. O tripwire de I/O é teste de regressão,
não uma sandbox contra código modificado.

```sh
python3 -I -S -B plugin.py
python3 -I -S -B -m unittest discover -s tests -v
```

No repositório Hub, siga o [guia completo](../../docs/public/STARTER.md) para
gerar outra identidade, registrar, visualizar capacidades, editar/reiniciar sem
recompilar e preparar uma submissão. `SETTINGS` e `INITIAL_VALUES` são o ponto
de partida; não anuncie hardware real no simulador.

Distribuir a pasta inteira conserva os testes e este guia relativo ao Hub;
`plugin.py` é autocontido e pode executar sem o restante do repositório.
