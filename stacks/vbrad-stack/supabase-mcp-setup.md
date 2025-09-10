User needs to provide:
    1. project ref (noted as id in workspace)
    2. access token

Example:

```.mcp.json
{
    "mcpServers": {
        "supabase": {
            "command": "npx",
            "args": [
                "-y",
                "@supabase/mcp-server-supabase",
                "--read-only",
                "--project-ref=ibtbbmqbbwytbquxodrj"  
            ],
            "env": {
                "SUPABASE_ACCESS_TOKEN": "sbp_fa03c247f5eed7ab2b5531645aea46f522900116"
            }
        }
    }
}
```