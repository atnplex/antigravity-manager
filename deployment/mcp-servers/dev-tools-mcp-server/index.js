#!/usr/bin/env node
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

const server = new Server(
  {
    name: "dev-tools",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  },
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "format-shell",
        description: "Format shell scripts using shfmt",
        inputSchema: {
          type: "object",
          properties: {
            code: { type: "string", description: "Shell code to format" },
            options: {
              type: "object",
              properties: {
                indent: { type: "number", default: 2 },
                binaryNextLine: { type: "boolean", default: false },
              },
            },
          },
          required: ["code"],
        },
      },
      {
        name: "format-python",
        description: "Format Python code using black",
        inputSchema: {
          type: "object",
          properties: {
            code: { type: "string", description: "Python code to format" },
            lineLength: { type: "number", default: 88 },
          },
          required: ["code"],
        },
      },
      {
        name: "format-json",
        description: "Format JSON using prettier",
        inputSchema: {
          type: "object",
          properties: {
            code: { type: "string", description: "JSON to format" },
            indent: { type: "number", default: 2 },
          },
          required: ["code"],
        },
      },
      {
        name: "format-typescript",
        description: "Format TypeScript/JavaScript using prettier",
        inputSchema: {
          type: "object",
          properties: {
            code: { type: "string", description: "TS/JS code to format" },
            semi: { type: "boolean", default: true },
            singleQuote: { type: "boolean", default: true },
          },
          required: ["code"],
        },
      },
      {
        name: "lint-shell",
        description: "Lint shell scripts using shellcheck",
        inputSchema: {
          type: "object",
          properties: {
            code: { type: "string", description: "Shell code to lint" },
          },
          required: ["code"],
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    if (name === "format-shell") {
      const { code, options = {} } = args;
      const indent = options.indent || 2;

      // Write to temp file, format, read back
      const { stdout } = await execAsync(
        `echo ${JSON.stringify(code)} | shfmt -i ${indent}`,
      );

      return {
        content: [
          {
            type: "text",
            text: stdout,
          },
        ],
      };
    }

    if (name === "format-python") {
      const { code, lineLength = 88 } = args;

      const { stdout } = await execAsync(
        `echo ${JSON.stringify(code)} | black -l ${lineLength} -`,
      );

      return {
        content: [
          {
            type: "text",
            text: stdout,
          },
        ],
      };
    }

    if (name === "format-json") {
      const { code, indent = 2 } = args;

      const formatted = JSON.stringify(JSON.parse(code), null, indent);

      return {
        content: [
          {
            type: "text",
            text: formatted,
          },
        ],
      };
    }

    if (name === "format-typescript") {
      const { code, semi = true, singleQuote = true } = args;

      const { stdout } = await execAsync(
        `echo ${JSON.stringify(code)} | prettier --parser typescript --semi=${semi} --single-quote=${singleQuote}`,
      );

      return {
        content: [
          {
            type: "text",
            text: stdout,
          },
        ],
      };
    }

    if (name === "lint-shell") {
      const { code } = args;

      try {
        const { stdout } = await execAsync(
          `echo ${JSON.stringify(code)} | shellcheck -`,
        );

        return {
          content: [
            {
              type: "text",
              text: stdout || "No issues found",
            },
          ],
        };
      } catch (err) {
        return {
          content: [
            {
              type: "text",
              text: err.stderr || err.message,
            },
          ],
        };
      }
    }

    throw new Error(`Unknown tool: ${name}`);
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error: ${error.message}`,
        },
      ],
      isError: true,
    };
  }
});

// Main
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("✓ Dev Tools MCP Server running");
}

main().catch(console.error);
