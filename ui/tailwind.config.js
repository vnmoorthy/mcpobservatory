/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["var(--font-sans)"],
        mono: ["var(--font-mono)"],
      },
      fontSize: {
        xs: "var(--fs-xs)",
        sm: "var(--fs-sm)",
        md: "var(--fs-md)",
        lg: "var(--fs-lg)",
        xl: "var(--fs-xl)",
      },
      colors: {
        bg0: "var(--bg-0)",
        bg1: "var(--bg-1)",
        bg2: "var(--bg-2)",
        bg3: "var(--bg-3)",
        border1: "var(--border)",
        fg0: "var(--fg-0)",
        fg1: "var(--fg-1)",
        fg2: "var(--fg-2)",
        accent: "var(--accent)",
        "accent-soft": "var(--accent-soft)",
        client: "var(--client-direction)",
        server: "var(--server-direction)",
        err: "var(--error)",
        "err-soft": "var(--error-soft)",
        warn: "var(--warning)",
        "warn-soft": "var(--warning-soft)",
        info: "var(--info)",
        "info-soft": "var(--info-soft)",
        purple: "var(--purple)",
        "purple-soft": "var(--purple-soft)",
      },
    },
  },
  plugins: [],
};
