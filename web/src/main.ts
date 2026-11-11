import { mount } from 'svelte'
import App from './App.svelte'
import { token } from './lib/api/token'
import './app.css'

token.adopt()
mount(App, { target: document.getElementById('app')! })
