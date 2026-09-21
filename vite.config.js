import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");

  return {
    server: {
      proxy: {
        "/graphql": `http://127.0.0.1:${env.API_PORT ?? 3000}`,
      },
    },
  };
});
