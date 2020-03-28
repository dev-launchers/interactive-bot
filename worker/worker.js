addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
  const { lottery, uuid } = wasm_bindgen;
  await wasm_bindgen(wasm)
  const botToken = await LOTTERY_BOT_CONFIG.get("token");
  const body = await request.json()
    .then(data => {
      return data;
    });

  const lotterResult = await lottery(body, botToken)
    .then(result => {
      return result
    })
    .catch(async function (err) {
      const sentryId = uuid();
      await sentryLog(err, sentryId);
    })

  return new Response("Hello, inquisitor", {
    "status": 200,
    "statusText": "OK",
    "headers": { 'Content-Type': 'text/plain' }
  })
}

async function sentryLog(err, id) {
  const currentTimestamp = Date.now() / 1000;
  const body = sentryEventJson(err, currentTimestamp, id);
  const sentryProectID = await SLACK_BRIDGE.get("sentryProjectID");
  const sentryKey = await SLACK_BRIDGE.get("sentryKey");
  return await fetch(`https://sentry.io/api/${sentryProectID}/store/`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Sentry-Auth': [
        'Sentry sentry_version=7',
        `sentry_timestamp=${currentTimestamp}`,
        `sentry_client=slack-bridge/0`,
        `sentry_key=${sentryKey}`
      ].join(', '),
    },
    body,
  });
}

function sentryEventJson(err, currentTimestamp, id) {
  return JSON.stringify({
    event_id: id,
    message: JSON.stringify(err),
    timestamp: currentTimestamp,
    logger: "slack-bridge-logger",
    platform: "javascript",
  })
}