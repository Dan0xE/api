# CodeDefender CLI

Example CodeDefender CLI usage:

```ps
$env:CD_API_KEY=eyJ0eX....
# Usage: codedefender-cli.exe --config <FILE> --api-key <API_KEY> --input-file <INPUT> --output <OUTPUT>

codedefender-cli.exe --config=.\example\config.yaml --input-file=.\example\HelloWorld.exe --pdb-file=.\example\HelloWorld.pdb --output output.zip
```

# Building

You can also build CodeDefender CLI for linux, MacOS, etc.