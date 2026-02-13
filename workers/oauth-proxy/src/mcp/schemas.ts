import * as v from 'valibot';

/**
 * JSON-RPC 2.0 request schema.
 * Used to extract method and params for analytics logging.
 */
export const JsonRpcRequestSchema = v.object({
  jsonrpc: v.literal('2.0'),
  method: v.string(),
  id: v.optional(v.union([v.string(), v.number(), v.null()])),
  params: v.optional(v.unknown()),
});

/**
 * MCP tools/call params schema.
 * Extracts tool name from tools/call requests.
 */
export const ToolCallParamsSchema = v.object({
  name: v.string(),
  arguments: v.optional(v.unknown()),
});

export interface ParsedJsonRpcRequest {
  method: string;
  toolName?: string;
  requestId?: string | number | null;
}

/**
 * Safely parse a JSON-RPC request body for analytics.
 * Returns null on any parsing failure (doesn't throw).
 *
 * @param body - Raw request body string
 * @returns Parsed request info or null if parsing fails
 */
export function parseJsonRpcRequest(body: string): ParsedJsonRpcRequest | null {
  try {
    const json: unknown = JSON.parse(body);
    const result = v.safeParse(JsonRpcRequestSchema, json);
    if (!result.success) return null;

    const { method, id, params } = result.output;
    let toolName: string | undefined;

    // Extract tool name for tools/call requests
    if (method === 'tools/call' && params) {
      const toolResult = v.safeParse(ToolCallParamsSchema, params);
      if (toolResult.success) {
        toolName = toolResult.output.name;
      }
    }

    return { method, toolName, requestId: id };
  } catch {
    return null;
  }
}
