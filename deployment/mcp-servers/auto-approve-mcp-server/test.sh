#!/bin/bash
# Test auto-approve MCP server

echo "=== Testing Auto-Approve MCP Server ==="
echo ""

echo "1. Testing file edit approval..."
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"check-approval","arguments":{"actionType":"file-edit","context":{"filePath":"/home/alex/projects/test.js","size":500}}}}' | node ~/auto-approve-mcp-server/index.js

echo ""
echo "2. Testing blocked file..."
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"check-approval","arguments":{"actionType":"file-edit","context":{"filePath":"/home/alex/.env","size":100}}}}' | node ~/auto-approve-mcp-server/index.js

echo ""
echo "3. Testing command approval..."
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"check-approval","arguments":{"actionType":"command","context":{"command":"git status"}}}}' | node ~/auto-approve-mcp-server/index.js

echo ""
echo "4. Testing blocked command..."
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"check-approval","arguments":{"actionType":"command","context":{"command":"rm -rf /"}}}}' | node ~/auto-approve-mcp-server/index.js

echo ""
echo "5. Testing MCP tool approval..."
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"check-approval","arguments":{"actionType":"mcp-tool","context":{"serverName":"github","toolName":"create-pr"}}}}' | node ~/auto-approve-mcp-server/index.js

echo ""
echo "✓ Tests complete. Check ~/.auto-approve-mcp/logs/ for detailed logs"
