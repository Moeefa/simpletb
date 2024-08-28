/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "media",
  content: ["./crates/ui/src/**/*.{html,js,tsx,ts,jsx}"],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
      },
    },
  },
  plugins: [],
};
