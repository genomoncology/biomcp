// Auto-generated slide navigation
(function() {
  var slides = [{% for s in slides %}'{{ s.slug }}'{% if not loop.last %},{% endif %}{% endfor %}];
  var deckId = '{{ deck_id | default("") }}';
  var current = location.pathname.split('/').pop().replace('.html', '');
  var idx = slides.indexOf(current);
  if (idx === -1) return;

  var PRES_KEY = 'deck-presenting';
  var inIframe = window.parent !== window;

  // ===== IFRAME PRESENTATION MODE =====
  if (inIframe && localStorage.getItem(PRES_KEY) === '1') {
    function applyScale() {
      var sx = window.innerWidth / 1280;
      var sy = window.innerHeight / 720;
      var s = Math.min(sx, sy);
      document.body.style.cssText = 'width:1280px;height:720px;transform:scale('+s+');transform-origin:top left;margin:0;position:absolute;left:'+Math.max(0,(window.innerWidth-1280*s)/2)+'px;top:'+Math.max(0,(window.innerHeight-720*s)/2)+'px;';
      document.documentElement.style.cssText = 'background:#1a1a1a;overflow:hidden;height:100%;';
    }
    applyScale();
    window.addEventListener('resize', applyScale);

    // Hide footer/page-num
    var f = document.querySelector('.footer');
    var p = document.querySelector('.page-num');
    if (f) f.style.display = 'none';
    if (p) p.style.display = 'none';

    // Navigation via postMessage
    function goNext() { if (idx < slides.length-1) window.parent.postMessage({action:'nav',slug:slides[idx+1]},'*'); }
    function goPrev() { if (idx > 0) window.parent.postMessage({action:'nav',slug:slides[idx-1]},'*'); }

    document.addEventListener('keydown', function(e) {
      if (e.key==='ArrowRight'||e.key===' ') { e.preventDefault(); goNext(); }
      else if (e.key==='ArrowLeft') { e.preventDefault(); goPrev(); }
      else if (e.key==='Escape'||e.key==='f'||e.key==='F') { e.preventDefault(); window.parent.postMessage({action:'exit'},'*'); }
    });

    var tx=0;
    document.addEventListener('touchstart', function(e){tx=e.changedTouches[0].screenX;},{passive:true});
    document.addEventListener('touchend', function(e){var d=e.changedTouches[0].screenX-tx;if(d<-60)goNext();else if(d>60)goPrev();},{passive:true});

    return; // no nav bar in iframe mode
  }

  // ===== NORMAL MODE =====
  var presenting = false;

  function goNext() { if (idx < slides.length-1) location.href = slides[idx+1]+'.html'; }
  function goPrev() { if (idx > 0) location.href = slides[idx-1]+'.html'; }
  function goIndex() { localStorage.removeItem(PRES_KEY); location.href = '/'+deckId+'/'; }
  function goHome() { localStorage.removeItem(PRES_KEY); location.href = '/'; }

  function enterPresentation() {
    presenting = true;
    localStorage.setItem(PRES_KEY, '1');
    bar.style.display = 'none';

    var overlay = document.createElement('div');
    overlay.id = 'pres-overlay';
    overlay.style.cssText = 'position:fixed;top:0;left:0;width:100vw;height:100vh;z-index:99999;background:#1a1a1a;';

    var frame = document.createElement('iframe');
    frame.id = 'pres-frame';
    frame.style.cssText = 'width:100%;height:100%;border:none;';
    frame.src = location.pathname;
    frame.onload = function() { try { frame.contentWindow.focus(); } catch(e){} };
    overlay.appendChild(frame);
    document.body.appendChild(overlay);

    if (overlay.requestFullscreen) {
      overlay.requestFullscreen().catch(function(){});
    }

    window.addEventListener('message', presHandler);
  }

  function presHandler(e) {
    if (!e.data || !e.data.action) return;
    if (e.data.action === 'nav') {
      var frame = document.getElementById('pres-frame');
      if (frame) {
        frame.src = '/'+deckId+'/'+e.data.slug+'.html';
        frame.onload = function() { try { frame.contentWindow.focus(); } catch(ex){} };
      }
    } else if (e.data.action === 'exit') {
      exitPresentation();
    }
  }

  function exitPresentation() {
    presenting = false;
    localStorage.removeItem(PRES_KEY);
    window.removeEventListener('message', presHandler);
    var overlay = document.getElementById('pres-overlay');
    if (overlay) overlay.remove();
    bar.style.display = 'flex';
    if (document.fullscreenElement) document.exitFullscreen().catch(function(){});
  }

  function togglePresentation() {
    if (presenting) exitPresentation(); else enterPresentation();
  }

  document.addEventListener('fullscreenchange', function() {
    if (!document.fullscreenElement && presenting) exitPresentation();
  });

  document.addEventListener('keydown', function(e) {
    if (presenting) return;
    if (e.key==='ArrowRight') goNext();
    else if (e.key==='ArrowLeft') goPrev();
    else if (e.key==='Escape') goIndex();
    else if (e.key==='f'||e.key==='F') togglePresentation();
  });

  var touchStartX=0;
  document.addEventListener('touchstart', function(e){touchStartX=e.changedTouches[0].screenX;},{passive:true});
  document.addEventListener('touchend', function(e){var d=e.changedTouches[0].screenX-touchStartX;if(d<-60)goNext();else if(d>60)goPrev();},{passive:true});

  // --- Navigation bar ---
  var bar = document.createElement('div');
  bar.style.cssText = 'position:fixed;bottom:0;left:0;right:0;height:48px;background:rgba(58,0,106,0.92);display:flex;align-items:center;padding:0 16px;z-index:9999;font-family:system-ui,sans-serif;gap:8px;';

  var homeBtn = document.createElement('button');
  homeBtn.innerHTML = '\u2302';
  homeBtn.title = 'All Decks';
  homeBtn.style.cssText = 'background:none;border:1px solid rgba(255,255,255,0.3);color:white;padding:8px 12px;border-radius:6px;font-size:16px;cursor:pointer;';
  homeBtn.onclick = goHome;

  var prevBtn = document.createElement('button');
  prevBtn.textContent = '\u25C0 Prev';
  prevBtn.style.cssText = 'background:none;border:1px solid rgba(255,255,255,0.3);color:white;padding:8px 16px;border-radius:6px;font-size:14px;cursor:pointer;'+(idx===0?'opacity:0.3;pointer-events:none;':'');
  prevBtn.onclick = goPrev;

  var spacerL = document.createElement('span');
  spacerL.style.cssText = 'flex:1;';
  var info = document.createElement('span');
  info.style.cssText = 'color:rgba(255,255,255,0.8);font-size:13px;cursor:pointer;';
  info.textContent = (idx+1)+' / '+slides.length;
  info.title = 'Deck index';
  info.onclick = goIndex;
  var spacerR = document.createElement('span');
  spacerR.style.cssText = 'flex:1;';

  var nextBtn = document.createElement('button');
  nextBtn.textContent = 'Next \u25B6';
  nextBtn.style.cssText = 'background:none;border:1px solid rgba(255,255,255,0.3);color:white;padding:8px 16px;border-radius:6px;font-size:14px;cursor:pointer;'+(idx===slides.length-1?'opacity:0.3;pointer-events:none;':'');
  nextBtn.onclick = goNext;

  var fullBtn = document.createElement('button');
  fullBtn.innerHTML = '\u25B6';
  fullBtn.title = 'Present (F)';
  fullBtn.style.cssText = 'background:none;border:1px solid rgba(255,255,255,0.3);color:white;padding:8px 12px;border-radius:6px;font-size:16px;cursor:pointer;margin-left:8px;';
  fullBtn.onclick = togglePresentation;

  bar.appendChild(homeBtn);
  bar.appendChild(prevBtn);
  bar.appendChild(spacerL);
  bar.appendChild(info);
  bar.appendChild(spacerR);
  bar.appendChild(nextBtn);
  bar.appendChild(fullBtn);
  document.body.appendChild(bar);

  var footer = document.querySelector('.footer');
  var pageNum = document.querySelector('.page-num');
  if (footer) footer.style.bottom = '60px';
  if (pageNum) pageNum.style.bottom = '60px';

})();
