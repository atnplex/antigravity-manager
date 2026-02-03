#!/usr/bin/env node
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CONFIG_PATH = path.join(
  process.env.HOME,
  ".auto-approve-mcp",
  "config.json",
);
const LOG_DIR = path.join(process.env.HOME, ".auto-approve-mcp", "logs");

// Default configuration
const DEFAULT_CONFIG = {
  rules: {
    fileEdits: {
      autoApprove: true,
      maxSize: 1048576, // 1MB
      allowedPaths: [
        `${process.env.HOME}/projects/*`,
        `${process.env.HOME}/*.md`,
      ],
      blockedPaths: ["/etc/*", "/sys/*", "*.env", "*credentials*", "*secrets*"],
    },
    commands: {
      autoApprove: true,
      allowList: [
        "git",
        "npm",
        "docker",
        "ls",
        "cat",
        "grep",
        "find",
        "cargo",
        "make",
      ],
      blockList: [
        "rm -rf /",
        "dd if=",
        "mkfs",
        "shutdown",
        "reboot",
        "systemctl stop",
      ],
    },
    mcpTools: {
      autoApprove: true,
      trustedServers: ["github", "docker", "filesystem", "git", "memory"],
    },
  },
  logging: {
    enabled: true,
    verbose: true,
  },
};

let config = DEFAULT_CONFIG;

// Load configuration
async function loadConfig() {
  try {
    const data = await fs.readFile(CONFIG_PATH, "utf8");
    config = JSON.parse(data);
    console.error("✓ Loaded config from", CONFIG_PATH);
  } catch (err) {
    console.error("! Using default config (no config file found)");
    await saveConfig();
  }
}

async function saveConfig() {
  try {
    await fs.mkdir(path.dirname(CONFIG_PATH), { recursive: true });
    await fs.writeFile(CONFIG_PATH, JSON.stringify(config, null, 2));
    console.error("✓ Saved config to", CONFIG_PATH);
  } catch (err) {
    console.error("! Failed to save config:", err.message);
  }
}

// Logging
async function logAction(action, context, approved, reason) {
  if (!config.logging.enabled) return;

  const timestamp = new Date().toISOString();
  const logEntry = {
    timestamp,
    action,
    context,
    approved,
    reason,
  };

  try {
    await fs.mkdir(LOG_DIR, { recursive: true });
    const logFile = path.join(
      LOG_DIR,
      `${new Date().toISOString().split("T")[0]}.jsonl`,
    );
    await fs.appendFile(logFile, JSON.stringify(logEntry) + "\n");

    if (config.logging.verbose) {
      console.error(
        `[${timestamp}] ${action}: ${approved ? "✓ APPROVED" : "✗ DENIED"} - ${reason}`,
      );
    }
  } catch (err) {
    console.error("! Failed to log action:", err.message);
  }
}

// Pattern matching
function matchesPattern(str, patterns) {
  return patterns.some((pattern) => {
    const regex = new RegExp(
      "^" + pattern.replace(/\*/g, ".*").replace(/\?/g, ".") + "$",
    );
    return regex.test(str);
  });
}

// Approval logic
function shouldAutoApprove(actionType, context) {
  if (actionType === "file-edit") {
    const { filePath, size } = context;

    // Check blocked paths first
    if (matchesPattern(filePath, config.rules.fileEdits.blockedPaths)) {
      return { approved: false, reason: "blocked path pattern" };
    }

    // Check file size
    if (size && size > config.rules.fileEdits.maxSize) {
      return {
        approved: false,
        reason: `file too large (${size} > ${config.rules.fileEdits.maxSize})`,
      };
    }

    // Check allowed paths
    if (config.rules.fileEdits.allowedPaths.length > 0) {
      if (!matchesPattern(filePath, config.rules.fileEdits.allowedPaths)) {
        return { approved: false, reason: "not in allowed paths" };
      }
    }

    return {
      approved: config.rules.fileEdits.autoApprove,
      reason: "file edit auto-approved",
    };
  }

  if (actionType === "command") {
    const { command } = context;

    // Check blocklist
    if (
      config.rules.commands.blockList.some((blocked) =>
        command.includes(blocked),
      )
    ) {
      return { approved: false, reason: "blocked command pattern" };
    }

    // Check allowlist
    const cmdName = command.trim().split(" ")[0];
    if (!config.rules.commands.allowList.includes(cmdName)) {
      return { approved: false, reason: "command not in allowlist" };
    }

    return {
      approved: config.rules.commands.autoApprove,
      reason: "command auto-approved",
    };
  }

  if (actionType === "mcp-tool") {
    const { serverName, toolName } = context;

    if (!config.rules.mcpTools.trustedServers.includes(serverName)) {
      return { approved: false, reason: "untrusted MCP server" };
    }

    return {
      approved: config.rules.mcpTools.autoApprove,
      reason: "trusted MCP tool",
    };
  }

  return { approved: false, reason: "unknown action type" };
}

// Create MCP server
const server = new Server(
  {
    name: "auto-approve",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  },
);

// List tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "check-approval",
        description:
          "Check if an action should be auto-approved based on rules",
        inputSchema: {
          type: "object",
          properties: {
            actionType: {
              type: "string",
              enum: ["file-edit", "command", "mcp-tool"],
              description: "Type of action to approve",
            },
            context: {
              type: "object",
              description: "Context for the action (filePath, command, etc.)",
            },
          },
          required: ["actionType", "context"],
        },
      },
      {
        name: "update-config",
        description: "Update auto-approve configuration",
        inputSchema: {
          type: "object",
          properties: {
            rules: {
              type: "object",
              description: "Updated rules configuration",
            },
          },
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  if (name === "check-approval") {
    const { actionType, context } = args;
    const result = shouldAutoApprove(actionType, context);

    await logAction(actionType, context, result.approved, result.reason);

    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  }

  if (name === "update-config") {
    const { rules } = args;
    config.rules = { ...config.rules, ...rules };
    await saveConfig();

    return {
      content: [
        {
          type: "text",
          text: "Configuration updated successfully",
        },
      ],
    };
  }

  throw new Error(`Unknown tool: ${name}`);
});

// Main
async function main() {
  await loadConfig();

  const transport = new StdioServerTransport();
  await server.connect(transport);

  console.error("✓ Auto-Approve MCP Server running");
  console.error(`  Config: ${CONFIG_PATH}`);
  console.error(`  Logs: ${LOG_DIR}`);
}

main().catch(console.error);
