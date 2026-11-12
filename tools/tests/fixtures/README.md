# Fixtures offline

- `publication-base.json`: metadados sintéticos válidos/incompletos; não representam periférico, pesquisa, evidência ou aprovação reais. A suíte não depende de documentos internos ou do exemplo local R1.
- `invalid-empty.json`: todos os metadados obrigatórios ausentes.
- `invalid-execution.json`: objeto incompleto e campo proibido de execução; seu texto é inerte.
- `invalid-cases.json`: mutações de um manifesto completo válido; paths são listas de chaves/índices, `value` é o valor inválido a inserir. Cada caso deve falhar no schema. Não são manifestos publicáveis.

Os testes criam temporariamente documentos/artefato de texto inerte, hashes calculados e URLs no domínio reservado example.org **somente como fixtures sintéticas**. Esse caso completo demonstra o gate `--submission` sem inventar release R1, autor real, evidência ou aprovação. Testes também geram symlinks, caminho inexistente/diretório, manifesto duplicado/NaN e hash incorreto. Nada é instalado, baixado ou executado como plugin.

Na v2 também há casos source-only válidos sem artefato, PNG RGB sintético de um pixel e segmentos JPEG sintéticos **somente para testar leitura de metadados**, versões SemVer/Git, datas comparadas ao calendário da biblioteca padrão e direitos de imagens incompletos. Os padrões do schema e o overlay `definitions/submission` são comparados ao resultado da CLI/biblioteca. O caso de dimensões grandes é framing de fixture, não imagem decodificada: não se afirma teste visual nem validade integral dos pixels comprimidos.

Regressões adicionais: submissão aceita `minimumHubVersion` e changelog ausentes ou `null`, mas recusa valores/referências inválidos quando declarados. `images.main` é opcional e pode usar arquivo igual ou diferente da thumbnail; seus caminhos, dimensões, bytes, formato, alt e direitos são conferidos independentemente, inclusive sem thumbnail.
