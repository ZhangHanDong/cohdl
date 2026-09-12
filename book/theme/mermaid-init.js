// 把 mdBook 渲染成普通代码块的 ```mermaid 围栏，在浏览器里换成 mermaid 图。
// 不依赖 mdbook-mermaid 预处理器，只需要同目录下本地打包的 mermaid.min.js。
(function () {
    var darkThemes = ['ayu', 'navy', 'coal'];
    var lightThemes = ['light', 'rust'];

    function isDark() {
        var classes = document.documentElement.classList;
        for (var i = 0; i < darkThemes.length; i++) {
            if (classes.contains(darkThemes[i])) { return true; }
        }
        return false;
    }

    function convertFences() {
        var blocks = document.querySelectorAll('pre > code.language-mermaid');
        var nodes = [];
        for (var i = 0; i < blocks.length; i++) {
            var code = blocks[i];
            var pre = code.parentElement;
            var target = document.createElement('pre');
            target.className = 'mermaid';
            target.textContent = code.textContent;
            pre.parentNode.replaceChild(target, pre);
            nodes.push(target);
        }
        return nodes;
    }

    function render() {
        if (typeof mermaid === 'undefined') { return; }
        var nodes = convertFences();
        if (!nodes.length) { return; }
        mermaid.initialize({
            startOnLoad: false,
            theme: isDark() ? 'dark' : 'default',
            securityLevel: 'strict'
        });
        mermaid.run({ nodes: nodes });
    }

    // 切换主题时重新加载页面，让图表按新主题重画；与 mdbook-mermaid 的做法一致。
    function hookThemeMenu() {
        var wasDark = isDark();
        darkThemes.forEach(function (id) {
            var el = document.getElementById(id);
            if (el) { el.addEventListener('click', function () { if (!wasDark) { window.location.reload(); } }); }
        });
        lightThemes.forEach(function (id) {
            var el = document.getElementById(id);
            if (el) { el.addEventListener('click', function () { if (wasDark) { window.location.reload(); } }); }
        });
    }

    function start() { render(); hookThemeMenu(); }
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', start);
    } else {
        start();
    }
})();
