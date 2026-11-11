import { mount } from 'svelte'
import App from './App.svelte'
import { token } from './lib/api/token'
import { nodrag } from './lib/ui/nodrag'
import './app.css'

token.adopt()
addEventListener('dragstart', nodrag)
mount(App, { target: document.getElementById('app')! })
