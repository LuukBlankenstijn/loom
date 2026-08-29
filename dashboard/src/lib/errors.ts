import { ConnectError } from "@connectrpc/connect";

/**
 * Extract a human-readable message from an error thrown by a gRPC-web call.
 * ConnectError prefixes its `message` with the status code (e.g.
 * "[already_exists] ..."); `rawMessage` gives just the server-provided text.
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof ConnectError) {
    return error.rawMessage || error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Something went wrong";
}
