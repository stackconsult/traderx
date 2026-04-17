# GitHub MCP Setup Guide

This document describes how to configure the GitHub Model Context Protocol (MCP) server for the TraderX project.

## Overview

The GitHub MCP server allows AI assistants to interact with the GitHub API for repository management, issue tracking, pull requests, and more.

**Repository:** https://github.com/stackconsult/traderx

## Prerequisites

1. Node.js 18+ installed
2. A GitHub account with access to the `stackconsult/traderx` repository
3. A GitHub Personal Access Token (PAT)

## Setup Instructions

### 1. Generate a GitHub Personal Access Token

1. Navigate to: https://github.com/settings/tokens
2. Click **"Generate new token"** → **"Generate new token (classic)"**
3. Provide a descriptive name (e.g., "TraderX MCP Access")
4. Select the following scopes:
   - `repo` - Full control of private repositories
   - `workflow` - Update GitHub Action workflows
   - `read:org` - Read organization data
5. Click **"Generate token"**
6. **Copy the token immediately** (you won't be able to see it again)

### 2. Configure Environment Variables

Copy the example environment file and add your token:

```bash
cp .env.example .env
```

Edit `.env` and replace:
```
GITHUB_TOKEN=ghp_your_actual_token_here
```

### 3. MCP Configuration

The MCP configuration is already set up at `.windsurf/mcp_config.json`. It references the `GITHUB_TOKEN` environment variable.

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${env:GITHUB_TOKEN}"
      }
    }
  }
}
```

### 4. Verify Installation

Restart Windsurf IDE to load the MCP configuration. The GitHub MCP tools should now be available.

## Available GitHub MCP Tools

Once configured, the following tools are available:

- `create_issue` - Create a new issue
- `update_issue` - Update an existing issue
- `search_issues` - Search for issues and PRs
- `create_pull_request` - Create a new PR
- `update_pull_request` - Update an existing PR
- `search_code` - Search code in the repository
- `list_commits` - List commits on a branch
- `get_file_contents` - Retrieve file contents
- `create_branch` - Create a new branch
- `list_branches` - List repository branches
- `search_repositories` - Search GitHub repositories

## Security Notes

- **Never commit the `.env` file** - It is already in `.gitignore`
- Use a dedicated PAT for MCP with minimal required scopes
- Rotate tokens periodically
- Store tokens securely; do not share them

## Troubleshooting

### MCP Server Not Starting

1. Verify Node.js is installed: `node --version`
2. Check that `GITHUB_TOKEN` is set in `.env`
3. Restart Windsurf IDE

### Authentication Errors

1. Verify your PAT has not expired
2. Check that the token has the required scopes
3. Ensure you have access to the `stackconsult/traderx` repository

## Reference

- [GitHub MCP Server Documentation](https://github.com/modelcontextprotocol/servers/tree/main/src/github)
- [GitHub Token Settings](https://github.com/settings/tokens)
