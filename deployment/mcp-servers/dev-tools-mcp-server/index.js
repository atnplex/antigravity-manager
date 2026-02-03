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

      // Use secure temp directory to avoid race conditions
      const fs = await import("fs/promises");
      const os = await import("os");
      const path = await import("path");
      const tempDir = await fs.mkdtemp(
        path.join(os.tmpdir(), "dev-tools-shfmt-"),
      );
      const tempFile = path.join(tempDir, "input.sh");

      try {
        await fs.writeFile(tempFile, code);
        const { stdout } = await execAsync(`shfmt -i ${indent} "${tempFile}"`);
        return {
          content: [
            {
              type: "text",
              text: stdout,
            },
          ],
        };
      } finally {
        await fs.rm(tempDir, { recursive: true, force: true }).catch(() => {});
      }
    }

    if (name === "format-python") {
      const { code, lineLength = 88 } = args;

      // Use secure temp directory to avoid race conditions
      const fs = await import("fs/promises");
      const os = await import("os");
      const path = await import("path");
      const tempDir = await fs.mkdtemp(
        path.join(os.tmpdir(), "dev-tools-black-"),
      );
      const tempFile = path.join(tempDir, "input.py");

      try {
        await fs.writeFile(tempFile, code);
        await execAsync(`black -l ${lineLength} "${tempFile}"`);
        const stdout = await fs.readFile(tempFile, "utf8");
        return {
          content: [
            {
              type: "text",
              text: stdout,
            },
          ],
        };
      } finally {
        await fs.rm(tempDir, { recursive: true, force: true }).catch(() => {});
      }
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

      // Use secure temp directory to avoid race conditions
      const fs = await import("fs/promises");
      const os = await import("os");
      const path = await import("path");
      const tempDir = await fs.mkdtemp(
        path.join(os.tmpdir(), "dev-tools-prettier-"),
      );
      const tempFile = path.join(tempDir, "input.ts");

      try {
        await fs.writeFile(tempFile, code);
        const { stdout } = await execAsync(
          `prettier --parser typescript --semi=${semi} --single-quote=${singleQuote} "${tempFile}"`,
        );
        return {
          content: [
            {
              type: "text",
              text: stdout,
            },
          ],
        };
      } finally {
        await fs.rm(tempDir, { recursive: true, force: true }).catch(() => {});
      }
    }

    if (name === "lint-shell") {
      const { code } = args;

      // Use secure temp directory to avoid race conditions
      const fs = await import("fs/promises");
      const os = await import("os");
      const path = await import("path");
      const tempDir = await fs.mkdtemp(
        path.join(os.tmpdir(), "dev-tools-shellcheck-"),
      );
      const tempFile = path.join(tempDir, "input.sh");

      try {
        await fs.writeFile(tempFile, code);
        const { stdout } = await execAsync(`shellcheck "${tempFile}"`);

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
      } finally {
        await fs.rm(tempDir, { recursive: true, force: true }).catch(() => {});
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
