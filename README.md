# Hevy Progressive Overloader

A webhook service for Hevy fitness app that automatically applies progressive overload to workout routines using AI.

## Environment Variables

### Required

- `HEVY_API_KEY` - Your Hevy API key
- `WEBHOOK_TOKEN` - Token for webhook authentication
- `GEMINI_API_KEY` - Google Gemini AI API key

### Optional

- `PORT` - Port to run the server on (default: 3005)
- `HEVY_API_URL` - Base URL for the Hevy API (default: https://api.hevyapp.com)
- `GEMINI_MODEL` - Gemini AI model to use (default: gemini-2.5-pro)

## Setup

1. Set up your environment variables:

```bash
export HEVY_API_KEY="your_hevy_api_key"
export WEBHOOK_TOKEN="your_webhook_token"
export GEMINI_API_KEY="your_gemini_api_key"
export PORT=3005  # Optional, defaults to 3005
```

2. Run the server:

```bash
cargo run
```

## API Endpoints

- `POST /webhook` - Webhook endpoint that processes workout completions
- Headers required: `Authorization: Bearer <WEBHOOK_TOKEN>`

## Logging

The service logs detailed information for debugging:

- **Workout processing** - When workouts are received and processed
- **AI prompts** - Full prompts sent to Gemini API
- **AI responses** - Raw JSON responses from Gemini
- **Routine updates** - Success/failure of routine modifications
- **Exercise suggestions** - Generated progressive overload recommendations

All logs use structured formatting for easy parsing and monitoring.
