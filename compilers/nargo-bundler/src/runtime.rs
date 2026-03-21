#![warn(missing_docs)]

use crate::types::FeatureSet;

/// 生成自定义运行时
pub fn generate_custom_runtime(features: &FeatureSet) -> String {
    let mut runtime = String::new();
    runtime.push_str("// Nargo Standalone Runtime\n\n");

    runtime.push_str("const runtime = (function() {\n");
    runtime.push_str("  const queue = [];\n");
    runtime.push_str("  let isFlushing = false;\n");
    runtime.push_str("  const p = Promise.resolve();\n");
    runtime.push_str("  function nextTick(fn) { return fn ? p.then(fn) : p; }\n");
    runtime.push_str("  function queueJob(job) { if (!queue.includes(job)) { queue.push(job); scheduleFlush(); } }\n");
    runtime.push_str("  function scheduleFlush() { if (!isFlushing) { isFlushing = true; nextTick(flushJobs); } }\n");
    runtime.push_str("  function flushJobs() { try { for (let i = 0; i < queue.length; i++) queue[i](); } finally { isFlushing = false; queue.length = 0; } }\n\n");

    runtime.push_str("  let currentEffect = null;\n");
    runtime.push_str("  function createSignal(v) {\n");
    runtime.push_str("    let val = v;\n");
    runtime.push_str("    const subs = new Set();\n");
    runtime.push_str("    return [\n");
    runtime.push_str("      () => { if (currentEffect) subs.add(currentEffect); return val; },\n");
    runtime.push_str("      (n) => { if (!Object.is(val, n)) { val = n; subs.forEach(s => queueJob(s)); } }\n");
    runtime.push_str(
        "    ];
",
    );
    runtime.push_str("  }\n\n");

    runtime.push_str("  function createEffect(fn) {\n");
    runtime.push_str("    const effect = () => {\n");
    runtime.push_str("      const prev = currentEffect;\n");
    runtime.push_str("      currentEffect = effect;\n");
    runtime.push_str("      try { fn(); } finally { currentEffect = prev; }\n");
    runtime.push_str(
        "    };
",
    );
    runtime.push_str("    effect();\n");
    runtime.push_str("  }\n\n");

    runtime.push_str("  function createComputed(fn) {\n");
    runtime.push_str("    const [g, s] = createSignal();\n");
    runtime.push_str("    createEffect(() => s(fn()));\n");
    runtime.push_str("    return g;\n");
    runtime.push_str("  }\n\n");

    runtime.push_str("  function h(tag, props, ...children) { return { tag, props, children: children.flat() }; }\n");
    runtime.push_str("  function createStaticVNode(html) { return { tag: 'div', props: { innerHTML: html }, children: [], isStatic: true }; }\n");

    runtime.push_str("  function mountElement(vnode, container) {\n");
    runtime.push_str("    if (typeof vnode === 'string' || typeof vnode === 'number') {\n");
    runtime.push_str("      const el = document.createTextNode(vnode);\n");
    runtime.push_str("      container.appendChild(el);\n");
    runtime.push_str("      return el;\n");
    runtime.push_str("    }\n");
    runtime.push_str("    if (vnode.isStatic) {\n");
    runtime.push_str("      const el = document.createElement(vnode.tag);\n");
    runtime.push_str("      el.innerHTML = vnode.props.innerHTML;\n");
    runtime.push_str("      const actualRoot = el.firstChild || el;\n");
    runtime.push_str("      container.appendChild(actualRoot);\n");
    runtime.push_str("      return actualRoot;\n");
    runtime.push_str("    }\n");
    runtime.push_str("    if (typeof vnode.tag === 'object') {\n");
    runtime.push_str("      return renderComponent(vnode.tag, container);\n");
    runtime.push_str("    }\n");
    runtime.push_str("    const el = document.createElement(vnode.tag);\n");
    runtime.push_str("    if (vnode.props) {\n");
    runtime.push_str("      for (const [key, value] of Object.entries(vnode.props)) {\n");
    runtime.push_str("        if (key.startsWith('on')) el.addEventListener(key.toLowerCase().slice(2), value);\n");
    runtime.push_str("        else el.setAttribute(key, value);\n");
    runtime.push_str("      }\n");
    runtime.push_str("    }\n");
    runtime.push_str("    vnode.children.forEach(child => mountElement(child, el));\n");
    runtime.push_str("    container.appendChild(el);\n");
    runtime.push_str("    return el;\n");
    runtime.push_str("  }\n\n");

    runtime.push_str("  function renderComponent(comp, container) {\n");
    runtime.push_str("    const setupContext = { i18n: comp.i18n || {} };\n");
    runtime.push_str("    const state = comp.setup ? comp.setup({}, setupContext) : {};\n");
    runtime.push_str("    let rootEl = null;\n");
    runtime.push_str("    createEffect(() => {\n");
    runtime.push_str("      const vnode = comp.render(state);\n");
    runtime.push_str("      if (rootEl) { container.removeChild(rootEl); }\n");
    runtime.push_str("      rootEl = mountElement(vnode, container);\n");
    runtime.push_str("    });\n");
    runtime.push_str("    return rootEl;\n");
    runtime.push_str("  }\n\n");

    runtime.push_str("  function useI18n(i18n) {\n");
    runtime.push_str("    return { t: (key, params) => {\n");
    runtime.push_str("      let msg = (i18n.en && i18n.en[key]) || key;\n");
    runtime.push_str("      if (params) Object.keys(params).forEach(k => msg = msg.replace(`{${k}}`, params[k]));\n");
    runtime.push_str("      return msg;\n");
    runtime.push_str("    }};\n");
    runtime.push_str("  }\n\n");

    runtime.push_str("  function lazy(loader) {\n");
    runtime.push_str("    let component = null;\n");
    runtime.push_str("    return {\n");
    runtime.push_str("      setup() {\n");
    runtime.push_str("        const [loading, setLoading] = createSignal(true);\n");
    runtime.push_str("        const [error, setError] = createSignal(null);\n");
    runtime.push_str("        \n");
    runtime.push_str("        loader().then(mod => {\n");
    runtime.push_str("          component = mod.default || mod;\n");
    runtime.push_str("          setLoading(false);\n");
    runtime.push_str("        }).catch(err => {\n");
    runtime.push_str("          setError(err);\n");
    runtime.push_str("          setLoading(false);\n");
    runtime.push_str("        });\n");
    runtime.push_str("        \n");
    runtime.push_str("        return {\n");
    runtime.push_str("          loading,\n");
    runtime.push_str("          error,\n");
    runtime.push_str("          component\n");
    runtime.push_str("        };");
    runtime.push_str("      },\n");
    runtime.push_str("      render({ loading, error, component }) {\n");
    runtime.push_str("        if (loading()) {\n");
    runtime.push_str("          return h('div', null, 'Loading...');\n");
    runtime.push_str("        }\n");
    runtime.push_str("        if (error()) {\n");
    runtime.push_str("          return h('div', null, `Error: ${error().message}`);\n");
    runtime.push_str("        }\n");
    runtime.push_str("        if (component) {\n");
    runtime.push_str("          return h(component);\n");
    runtime.push_str("        }\n");
    runtime.push_str("        return null;\n");
    runtime.push_str("      }\n");
    runtime.push_str("    };");
    runtime.push_str("  }\n\n");

    runtime.push_str(
        "  return { 
",
    );
    runtime.push_str(
        "    signal: createSignal, createSignal, 
",
    );
    runtime.push_str(
        "    effect: createEffect, createEffect, 
",
    );
    runtime.push_str(
        "    computed: createComputed, createComputed, 
",
    );
    runtime.push_str(
        "    nextTick, h, render: renderComponent, useI18n, createStaticVNode, lazy 
",
    );
    runtime.push_str(
        "  };
",
    );
    runtime.push_str("})();\n\n");

    runtime.push_str("const { signal, createSignal, effect, createEffect, computed, createComputed, nextTick, h, render, useI18n, createStaticVNode, lazy } = runtime;\n");
    runtime.push_str("const t = useI18n({}).t;\n");

    runtime
}
