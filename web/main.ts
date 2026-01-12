import { mount } from 'svelte'
import './style.css'
import App from './App.svelte'
import { themeManager } from './lib/theme.svelte'

// Initialize theme early
const _ = themeManager.theme;

mount(App, {
  target: document.getElementById('app')!,
})
