#!/bin/bash
# Don't redirect stderr - MCP protocol needs it!
cd /home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh
export PYTHONPATH=/home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh:${PYTHONPATH}
exec /home/papadoc/Gooseprojects/goose-org-twin/.venv-agent-mesh/bin/python3 -u -m agent_mesh_server "$@"
