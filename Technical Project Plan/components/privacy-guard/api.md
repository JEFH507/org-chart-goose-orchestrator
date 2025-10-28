# API

## MCP tools
- privacy_guard.detect {text} → {entities:[{type,start,end,score}]}
- privacy_guard.process {text, policy:{maskClasses:["PII","SECRETS"], mode:"mask"}} → {masked,mapRef}

## Provider wrapper
- Pre-call: process(); Post-call: reidentify() if allowed.
