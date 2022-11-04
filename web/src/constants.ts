import { env } from '$env/dynamic/public';

export const APP_HOST: string = env.APP_HOST ? env.APP_HOST : 'localhost';
export const APP_PORT: string = env.APP_PORT ? env.APP_PORT : '8080';
// export const APP_PORT: string = env.APP_PORT;
export const APP_REMOTE_URL: string = env.APP_REMOTE_URL
	? env.APP_REMOTE_URL
	: `${APP_HOST}:${APP_PORT}`;
