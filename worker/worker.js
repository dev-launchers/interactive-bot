addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
  const { lottery, post_message_future } = wasm_bindgen;
  await wasm_bindgen(wasm)
  const botToken = await LOTTERY_BOT_CONFIG.get("token");
  const body = await request.json()
    .then(data => {
      return data;
    });
  console.log("body", JSON.stringify(body));
  const lotterResult = await lottery(body, botToken);
  console.log("lotteryResult", lotterResult);
  //const greeting = greet(botToken)
  //await post_message_future(botToken);
  return new Response("Hello, inquisitor", {
    "status": 200,
    "statusText": "OK",
    "headers": { 'Content-Type': 'text/plain' }
  })
}
