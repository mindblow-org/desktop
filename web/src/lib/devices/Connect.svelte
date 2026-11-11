<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { encode } from 'uqr'
  import { api } from '../api/http'
  import { token } from '../api/token'
  import { device } from '../local'
  import { store } from '../notes/store.svelte'
  import { ago } from '../ui/ago'
  import Icon from '../ui/Icon.svelte'
  import { clock } from '../ui/now.svelte'
  import { name } from './name'

  let { onclose }: { onclose: () => void } = $props()
  let el: HTMLDivElement
  let field = $state<HTMLInputElement>()
  let base = $state<string | null>()
  let copied = $state(false)
  let naming = $state(false)
  let draft = $state('')
  const me = device()
  const secret = token.get()
  const local = ['localhost', '127.0.0.1', '[::1]'].includes(location.hostname)
  const qr = $derived(base ? draw(`${base}/${secret ? `#token=${encodeURIComponent(secret)}` : ''}`) : null)

  onMount(() => {
    if (!local) base = location.origin
    else
      api
        .health()
        .then((h) => (base = h.link))
        .catch(() => (base = null))
    const key = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      e.stopPropagation()
      if (naming) naming = false
      else onclose()
    }
    const away = (e: PointerEvent) => {
      const t = e.target as HTMLElement
      if (!el.contains(t) && !t.closest('.qrw')) onclose()
    }
    addEventListener('keydown', key, true)
    addEventListener('pointerdown', away, true)
    return () => {
      removeEventListener('keydown', key, true)
      removeEventListener('pointerdown', away, true)
    }
  })

  function draw(text: string) {
    const q = encode(text, { ecc: 'M', border: 0 })
    let d = ''
    q.data.forEach((row, y) => {
      for (let x = 0; x < row.length; x++) {
        if (!row[x]) continue
        let n = 1
        while (row[x + n]) n++
        d += `M${x} ${y}h${n}v1h-${n}z`
        x += n - 1
      }
    })
    return { d, size: q.size }
  }

  async function copy() {
    if (!base) return
    try {
      await navigator.clipboard.writeText(base)
    } catch {
      const t = document.createElement('textarea')
      t.value = base
      document.body.append(t)
      t.select()
      document.execCommand('copy')
      t.remove()
    }
    copied = true
    setTimeout(() => (copied = false), 1600)
  }

  async function rename() {
    draft = name.get()
    naming = true
    await tick()
    field?.select()
  }

  function save() {
    if (!naming) return
    naming = false
    store.rename(draft)
  }
</script>

<div class="pop" bind:this={el} role="dialog" aria-label="Conectar outro aparelho">
  <div class="head"><span>conectar outro aparelho</span><span>esc fecha</span></div>
  {#if qr && base}
    <div class="qr">
      <svg viewBox="0 0 {qr.size} {qr.size}" shape-rendering="crispEdges" role="img" aria-label="QR code do link do mindblow"><path d={qr.d} /></svg>
    </div>
    <div class="link">
      <span>{base.replace(/^https?:\/\//, '')}</span>
      <button class="ib" title="Copiar link" aria-label="Copiar link" onclick={copy}>
        {#if copied}<span class="ok">copiado</span>{:else}<Icon name="copy" size={14} />{/if}
      </button>
    </div>
    {#if secret}<p class="note">O QR leva o token: quem escanear entra direto, sem digitar.</p>{/if}
  {:else if base === null}
    <p class="note none">Sem o link da tailnet. Abra o mindblow pelo link da tailnet, ou rode <code>./deploy/install.sh</code> de novo para o servidor saber qual é.</p>
  {:else}
    <div class="qr wait"></div>
  {/if}

  <div class="head devs">
    <span>dispositivos conectados</span>
    {#if store.online}<span>{store.devices.length}</span>{:else}<span class="off">offline · reconectando…</span>{/if}
  </div>
  <ul>
    {#each store.devices as d (d.id)}
      <li>
        <i></i>
        {#if d.id === me && naming}
          <input
            bind:this={field}
            bind:value={draft}
            maxlength="40"
            placeholder={d.name}
            aria-label="Nome deste aparelho"
            onblur={save}
            onkeydown={(e) => e.key === 'Enter' && save()}
          />
        {:else}
          <span class="n">{d.name || 'sem nome'}</span>
          {#if d.id === me}
            <span class="me">este</span>
            <button class="ib edit" title="Renomear este aparelho" aria-label="Renomear este aparelho" onclick={rename}><Icon name="pen" size={12} /></button>
          {/if}
        {/if}
        <span class="t">{ago(d.since, clock.now)}</span>
      </li>
    {:else}
      <li class="empty">ninguém conectado</li>
    {/each}
  </ul>
</div>

<style>
  .pop {
    position: absolute;
    top: 38px;
    right: -6px;
    z-index: 50;
    width: 300px;
    padding: 14px;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-pop);
    transform-origin: calc(100% - 20px) 0;
    animation: in 140ms var(--ease);
  }

  @keyframes in {
    from {
      opacity: 0;
      scale: 0.96;
    }
  }

  .head {
    display: flex;
    justify-content: space-between;
    margin: 0 2px 10px;
    font: 11px var(--mono);
    color: var(--faint);
  }

  .devs {
    margin-top: 16px;
  }

  .off {
    color: var(--tone-1);
  }

  .qr {
    display: grid;
    place-items: center;
    width: 200px;
    aspect-ratio: 1;
    margin: 0 auto;
    padding: 14px;
    border-radius: var(--r-md);
    background: #fff;
  }

  .qr svg {
    display: block;
    width: 100%;
    height: 100%;
    fill: #111;
  }

  .wait {
    background: var(--hover);
  }

  .link {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    margin-top: 10px;
    font: 11.5px var(--mono);
    color: var(--soft);
  }

  .link > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ok {
    font: 10.5px var(--mono);
    color: var(--fg);
  }

  .note {
    margin: 8px 4px 0;
    text-align: center;
    font-size: 11.5px;
    line-height: 1.45;
    color: var(--muted);
  }

  .none {
    margin-top: 0;
    padding: 14px;
    border-radius: var(--r-md);
    background: var(--hover);
  }

  code {
    font: 11px var(--mono);
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    padding: 0 6px;
    border-radius: var(--r-sm);
    font-size: 13px;
  }

  li:hover {
    background: var(--hover);
  }

  li > i {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--tone-3);
  }

  .n {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .me {
    flex: none;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--hover);
    font: 10px var(--mono);
    color: var(--muted);
  }

  li:hover .me {
    background: var(--paper);
  }

  .edit {
    min-width: 24px;
    height: 24px;
    padding: 0;
  }

  .t {
    flex: none;
    margin-left: auto;
    font: 10.5px var(--mono);
    color: var(--faint);
  }

  input {
    flex: 1;
    min-width: 0;
    height: 26px;
    padding: 0 8px;
    border-radius: var(--r-sm);
    box-shadow: inset 0 0 0 1px var(--line-strong);
    font-size: 13px;
  }

  .empty {
    justify-content: center;
    font: 11px var(--mono);
    color: var(--faint);
  }

  @media (max-width: 640px) {
    .pop {
      position: fixed;
      top: auto;
      left: 12px;
      right: 12px;
      bottom: calc(var(--dock) + 10px + env(safe-area-inset-bottom));
      width: auto;
      transform-origin: 50% 100%;
    }
  }
</style>
