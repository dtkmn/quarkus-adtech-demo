import http from 'k6/http';
import { check } from 'k6';
import { randomIntBetween, randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

export const options = {
    vus: 50,           // 50 concurrent users
    duration: '30s',   // for 30 seconds
};

const domains = ['espn.com', 'cnn.com', 'nytimes.com', 'reddit.com'];
const uas = ['Mozilla/5.0 (iPhone)', 'Mozilla/5.0 (Macintosh)', 'Mozilla/5.0 (Linux; Android 13)'];

export default function () {
    const port = __ENV.TARGET_PORT || '8070'; // Default to 8070 if not set
    const payload = JSON.stringify({
        id: `req-${randomIntBetween(1, 999999)}`,
        site: { id: "site-1", domain: randomItem(domains) },
        device: { ip: "1.2.3.4", ua: randomItem(uas), lmt: randomIntBetween(0, 1) }
    });
    const params = { headers: { 'Content-Type': 'application/json' } };

    const res = http.post(`http://localhost:${port}/bid-request`, payload, params);
    // if (res.status !== 200) {
    //     console.log(`Got status: ${res.status}`);
    // }
    // check(res, { 'is status 200': (r) => r.status === 200 });
    // 200 = We bid. 204 = We successfully filtered it out. Both are GOOD.
    check(res, {
        'is legitimate response (200 or 204)': (r) => r.status === 200 || r.status === 204,
    });
}
