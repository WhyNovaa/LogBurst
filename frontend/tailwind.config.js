/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f2fcf5',
          100: '#e1f8e8',
          200: '#c3efd4',
          300: '#94e0b5',
          400: '#5cc992',
          500: '#34af75',
          600: '#268e5d',
          700: '#23724d',
          800: '#205a3f',
          900: '#1b4a35',
          950: '#0e291e',
        },
        beige: {
          50: '#fbfaf8',
          100: '#f6f3ee',
          200: '#ede6db',
          300: '#dfd1be',
          400: '#cbb69b',
          500: '#b89a7d',
          600: '#aa8669',
          700: '#8e6b54',
          800: '#755848',
          900: '#5f493d',
          950: '#33261f',
        }
      }
    },
  },
  plugins: [],
}
