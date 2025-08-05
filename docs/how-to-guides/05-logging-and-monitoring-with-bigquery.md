# BigQuery Logging for BioMCP

This document outlines how BioMCP uses Google BigQuery for logging user interactions and API usage.

## Overview

BioMCP integrates with Google BigQuery to log user interactions, queries, and API usage. This logging provides valuable insights into how the system is being used, helps with debugging, and enables analytics for improving the service.

## Prerequisites

- A Google Cloud Platform (GCP) account
- A BigQuery dataset and table created in your GCP project
- A GCP service account with BigQuery permissions

## Setting Up BigQuery for BioMCP

1. **Create a BigQuery Dataset and Table**

   - In the Google Cloud Console, navigate to BigQuery
   - Create a new dataset (e.g., `biomcp_logs`)
   - Create a table within the dataset (e.g., `worker_logs`) with the following schema:
     ```
     timestamp: TIMESTAMP
     userEmail: STRING
     query: STRING
     ```
   - Adjust the schema as needed for your specific logging requirements

2. **Create a Service Account**

   - Navigate to "IAM & Admin" > "Service Accounts" in the Google Cloud Console
   - Create a new service account with a descriptive name (e.g., `biomcp-bigquery-logger`)
   - Assign the "BigQuery Data Editor" role to the service account
   - Create and download a JSON key for the service account

3. **Configure BioMCP with BigQuery Credentials**

   - Open `wrangler.toml` in the BioMCP project
   - Update the following variables with your BigQuery information:
     ```toml
     BQ_PROJECT_ID = "your-gcp-project-id"
     BQ_DATASET = "biomcp_logs"
     BQ_TABLE = "worker_logs"
     ```
   - For the service account key, use Cloudflare's secret management:
     ```bash
     npx wrangler secret put BQ_SA_KEY_JSON
     ```
     When prompted, paste the entire JSON content of your service account key file

## How BigQuery Logging Works

The BioMCP worker uses the following process to log data to BigQuery:

1. **Authentication**: The worker generates a JWT token using the service account credentials
2. **Token Exchange**: The JWT is exchanged for a Google OAuth access token
3. **Data Insertion**: The worker uses BigQuery's streaming insert API to log events

The implementation includes:

- Token caching to minimize authentication requests
- Error handling for failed logging attempts
- Automatic retry logic for transient failures

## Logged Information

By default, the following information is logged to BigQuery:

- **timestamp**: When the event occurred
- **userEmail**: The email address of the authenticated user (if available)
- **query**: The query or request that was made

You can extend the logging schema to include additional information as needed.

## Accessing and Analyzing Logs

To access and analyze the logs:

1. **Query the BigQuery Table**

   - Use the BigQuery console or SQL to query your logs
   - Example query to see recent logs:
     ```sql
     SELECT timestamp, userEmail, query
     FROM `your-project.biomcp_logs.worker_logs`
     ORDER BY timestamp DESC
     LIMIT 100
     ```

2. **Create Visualizations**

   - Use Google Data Studio to create dashboards based on your BigQuery data
   - Connect Data Studio to your BigQuery table and create visualizations

## Security Considerations

- The service account key is sensitive information and should be protected
- Use Cloudflare's secret management to store the key securely
- Consider implementing field-level encryption for sensitive data
- Implement data retention policies to comply with privacy regulations
- **IMPORTANT: Never include PHI (Protected Health Information) or PII (Personally Identifiable Information) in queries or logs**
  - Ensure all queries are sanitized to remove patient identifiers, medical record numbers, and other sensitive information
  - Consider implementing automatic redaction of potential PHI/PII from logs
  - Regularly audit logs to ensure compliance with HIPAA and other privacy regulations
  - Remember that BigQuery logs are not designed for storing protected health information

### Automatic Sanitization

BioMCP automatically sanitizes sensitive data before logging to BigQuery:

- **API Keys and Secrets**: Fields containing `api_key`, `apiKey`, `api-key`, `token`, `secret`, or `password` are automatically redacted
- **Nested Objects**: Sanitization works recursively through nested objects and arrays
- **Case-Insensitive**: Field name matching is case-insensitive to catch variations
- **Preserved Structure**: The original request structure is maintained with sensitive values replaced by `[REDACTED]`

Example of sanitization:

```javascript
// Original request
{
  "params": {
    "arguments": {
      "api_key": "AIzaSyB1234567890",
      "gene": "BRAF"
    }
  }
}

// Sanitized for BigQuery
{
  "params": {
    "arguments": {
      "api_key": "[REDACTED]",
      "gene": "BRAF"
    }
  }
}
```

### Excluded Queries

Certain types of queries are automatically excluded from BigQuery logging:

- **Think Tool Calls**: Any calls to the `think` tool are not logged
- **Thinking Domain**: Queries with `domain="thinking"` or `domain="think"` are excluded
- **Privacy-First Design**: This ensures that internal reasoning and analysis steps remain private

## Troubleshooting

- **Authentication Failures**: Verify that the service account key is correctly formatted and has the necessary permissions
- **Insertion Errors**: Check that the BigQuery table schema matches the data being inserted
- **Missing Logs**: Ensure that the worker has network access to the BigQuery API

## Example Code

The worker includes the following key functions for BigQuery logging:

- `getBQToken()`: Fetches and caches a BigQuery OAuth token
- `insertEvent()`: Inserts a single row into BigQuery via streaming insert
- `sanitizeObject()`: Recursively sanitizes sensitive fields from objects before logging

These functions handle the authentication and data insertion process automatically.

## Testing

BioMCP includes comprehensive tests for the BigQuery logging functionality:

### JavaScript Tests

The sanitization logic is tested using Node.js built-in test framework:

```bash
# Run JavaScript worker tests
make test-js

# Or run directly
node --test tests/tdd/workers/test_worker_sanitization.js
```

Tests cover:

- API key redaction
- Nested sensitive field handling
- Array sanitization
- Case-insensitive field matching
- Think tool detection
- Domain-based filtering
