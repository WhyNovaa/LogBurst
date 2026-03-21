import http from 'k6/http';
import {check} from 'k6';

export const options = {
    // Оптимизация: не тратим память и CPU на обработку тела ответа
    // (нам важен только статус код). Это позволяет выжать больше RPS из генератора.
    discardResponseBodies: true,

    scenarios: {
        stress_test: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                {duration: '15s', target: 1000},  // Попытка выдавить максимум (экстрим)
                {duration: '15s', target: 0},      // Остывание
            ],
            gracefulRampDown: '10s',
        },
    },

    thresholds: {
        http_req_failed: ['rate<0.01'], // Ошибок не больше 1%
        http_req_duration: ['p(95)<500'], // 95% запросов быстрее 500мс
    },
};

// --- ОПТИМИЗАЦИЯ: Инициализация данных один раз ---

// Если запускаете локально без Docker, поменяйте на http://localhost:8080/logs
const url = 'http://0.0.0.0:8080/v1/logs';

// Сериализуем JSON один раз при старте, чтобы не тратить CPU в цикле
const payload = JSON.stringify({
    timestamp: 1770151080,
    level: 'error',
    service: 'aboba',
    message: 'huy',
});

const params = {
    headers: {
        'Content-Type': 'application/json',
        'X-SERVICE-ID': 'service-a',
        'X-SIGNATURE': '153fdadb3266d0579b2d2ad0888ac49e26d6a64f4ecfac0cadb632b1479481a0'
    },
};

// --- Основной цикл ---
export default function () {
    const res = http.post(url, payload, params);

    check(res, {
        'status was 200': (r) => r.status == 200,
    });
}
