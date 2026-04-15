# GitHub MCP Server Setup & Configuration

## Server Configuration

**File**: `c:\Users\Geoff Parsons\.codeium\windsurf\mcp_config.json`

**Status**: ✅ Updated with new token

```json
{
  "mcpServers": {
    "io.windsurf/github-mcp-server": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "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",
        "github_token": "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"
      },
      "disabled": false,
      "registry": "io.windsurf/github-mcp-server"
    }
  }
}
```

## Requirements

**Docker**: Required to run the GitHub MCP server container
**Token**: Must have `repo` scope for private repositories

## How It Works

1. Docker pulls `ghcr.io/github/github-mcp-server`
2. Container runs with provided GitHub token
3. Provides MCP tools for GitHub operations:
   - List repositories
   - Get Actions runs
   - Create PRs
   - Merge PRs
   - Create issues
   - And more...

## Validation Approach

Since the MCP server provides GitHub tools, I should be able to:
1. Query Actions status via MCP tools
2. Monitor workflow runs
3. Get real-time status
4. Report back to user

## Current Status

**Token**: Updated ✅  
**Config**: Valid ✅  
**Docker**: Must be running  
**Server**: Should start automatically when tools are called

## Next Step

Attempt to use GitHub MCP tools to query Actions status...
