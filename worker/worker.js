addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
  const { lottery } = wasm_bindgen;
  await wasm_bindgen(wasm)
  const botToken = await LOTTERY_BOT_CONFIG.get("token");
  const body = await request.json()
    .then(data => {
      return data;
    });
  const lotterResult = await lottery(body, botToken);
  console.log("lotteryResult", lotterResult);
  return new Response("Hello, inquisitor", {
    "status": 200,
    "statusText": "OK",
    "headers": { 'Content-Type': 'text/plain' }
  })
}
