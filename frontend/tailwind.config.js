/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        'gv-dark': '#0a0a0f',
        'gv-card': '#14141f',
        'gv-hover': '#1f1f2e',
        'gv-accent': '#6366f1',
        'gv-success': '#22c55e',
        'gv-warning': '#f59e0b',
      },
    },
  },
  plugins: [],
}
