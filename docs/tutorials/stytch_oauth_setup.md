# Setting Up Stytch OAuth for BioMCP

This document outlines the steps required to set up Stytch OAuth authentication for the BioMCP application.

## Prerequisites

- A Stytch account (sign up at [Stytch](https://stytch.com/))
- Access to the BioMCP codebase

## Setting Up Your Stytch Account

1. **Create a Stytch Account**

   - Sign up at [Stytch](https://stytch.com/)
   - Select "Consumer Authentication" when prompted during the setup process

2. **Configure the Stytch Project**

   - In the Stytch dashboard, navigate to "Frontend SDKs" and enable the frontend SDK
   - Navigate to "Connected Apps" and enable "Dynamic Client Registration"
   - This allows MCP clients to register themselves dynamically with Stytch

3. **Retrieve Your Credentials**
   - Go to "Project Settings" in the Stytch dashboard
   - Note down the following credentials:
     - Project ID
     - Secret (API Key)
     - Public Token

## Configuring BioMCP with Stytch

1. **Update the Wrangler Configuration**

   - Open `wrangler.toml` in the BioMCP project
   - Update the following variables with your Stytch credentials:
     ```toml
     STYTCH_PROJECT_ID = "your-project-id"
     STYTCH_SECRET = "your-secret-key"
     STYTCH_PUBLIC_TOKEN = "your-public-token"
     ```
   - For development, use the test environment:
     ```toml
     STYTCH_API_URL = "https://test.stytch.com/v1"
     ```
   - For production, use:
     ```toml
     STYTCH_API_URL = "https://api.stytch.com/v1"
     ```

2. **Configure the OAuth KV Namespace**

   - Create a KV namespace in Cloudflare for storing OAuth tokens and state
   - Update the KV namespace ID in `wrangler.toml`:
     ```toml
     [[kv_namespaces]]
     binding = "OAUTH_KV"
     id = "your-kv-namespace-id"
     ```

3. **Configure JWT Secret**
   - Set a strong JWT secret for token signing:
     ```toml
     JWT_SECRET = "your-secure-jwt-secret"
     ```

## OAuth Flow Overview

The BioMCP application uses the following OAuth flow:

1. **Discovery**: MCP clients fetch OAuth metadata to locate Stytch authorization endpoints
2. **Registration**: MCP clients dynamically register with Stytch
3. **Authorization**: Users are redirected to Stytch for authentication and consent
4. **Token Exchange**: After consent, authorization codes are exchanged for access tokens
5. **MCP Connection**: MCP clients connect to the BioMCP server using OAuth access tokens

## Endpoints

The worker implements the following OAuth endpoints:

- `/.well-known/oauth-authorization-server`: OAuth server metadata
- `/authorize`: OAuth authorization endpoint
- `/callback`: OAuth callback endpoint
- `/token`: Token exchange endpoint

## Integration with BigQuery Logging

BioMCP integrates Stytch OAuth authentication with BigQuery logging to track user interactions and API usage. This integration:

1. **Captures Authentication Events**: Logs successful and failed authentication attempts
2. **Tracks User Activity**: Associates API requests with authenticated users
3. **Enables Analytics**: Provides insights into how users interact with the system

For detailed information about the BigQuery logging setup and usage, see the [BigQuery Logging Documentation](/docs/tutorials/bigquery_logging.md).

## Testing

To test the OAuth implementation:

1. Deploy the worker to Cloudflare:

   ```
   wrangler deploy
   ```

2. Use the MCP Inspector or another OAuth client to test the flow:
   - Set the OAuth discovery URL to: `https://your-worker.workers.dev/.well-known/oauth-authorization-server`
   - The inspector will guide you through the OAuth flow

## Troubleshooting

- **JWT Validation Issues**: Ensure the JWKS endpoint is correctly configured and accessible
- **Callback Errors**: Check that the redirect URIs are properly registered and match exactly
- **Token Exchange Failures**: Verify that the authorization code is valid and not expired

## Security Considerations

- Always use HTTPS for all OAuth endpoints
- Implement proper CORS headers for cross-origin requests
- Regularly rotate the JWT secret
- Use the production Stytch API URL for production environments
