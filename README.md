# Hevy Progressive Overloader

A webhook service for Hevy fitness app that automatically applies progressive overload to workout routines using AI.

## Environment Variables

### Required

- `HEVY_API_KEY` - Your Hevy API key
- `WEBHOOK_TOKEN` - Token for webhook authentication
- `GEMINI_API_KEY` - Google Gemini AI API key
- `PORT` - Port to run the server on (default: 3005)

### Optional

- `USE_MOCK_GEMINI=true` - Use the actual Gemini API instead of mock responses
- `BASE_URL` - Base URL for the Hevy API (default: <https://api.hevyapp.com>)

## Setup

1. Set up your environment variables:

```bash
export HEVY_API_KEY="your_hevy_api_key"
export WEBHOOK_TOKEN="your_webhook_token"
export GEMINI_API_KEY="your_gemini_api_key"
export PORT=3005
```

2. Run the server:

```bash
cargo run
```

## Mock Response Mode

By default, the service uses a hardcoded mock response for Gemini API calls to avoid consuming API tokens during development. To use the real Gemini API:

```bash
USE_MOCK_GEMINI=true cargo run
```

## API Endpoints

- `POST /webhook` - Webhook endpoint that processes workout completions
- Headers required: `Authorization: Bearer <WEBHOOK_TOKEN>`

## Response Logging

The service logs:

- **GEMINI PROMPT** - The full prompt sent to Gemini
- **GEMINI RESPONSE** - The raw JSON response from Gemini

These are useful for debugging and can be captured for testing purposes.
