import { useState } from "react";

const T = {
  bg: "#3B2FE0", card: "#FFFFFF", cardSurface: "#F4F4F8",
  accent: "#F59E0B", accentLight: "#FEF3C7",
  success: "#10B981", danger: "#EF4444",
  text: "#1A1A2E", textSec: "#6B7280", textMuted: "#9CA3AF",
  border: "#E8E8EE",
};

const I = ({ n, s = 20, c = "currentColor" }) => {
  const p = {
    heart: <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>,
    users: <><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></>,
    hand: <><path d="M18 11V6a2 2 0 0 0-4 0v5"/><path d="M14 10V4a2 2 0 0 0-4 0v6"/><path d="M10 9.5V5a2 2 0 0 0-4 0v9"/><path d="M22 11v1a8 8 0 0 1-8 8h-2c-2.5 0-4-1-6-3l-3-3a2 2 0 0 1 2.8-2.8L8 13"/><path d="M18 8a2 2 0 1 1 4 0v3"/></>,
    user: <><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></>,
    plus: <><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></>,
    chevL: <polyline points="15 18 9 12 15 6"/>,
    chevR: <polyline points="9 18 15 12 9 6"/>,
    gift: <><polyline points="20 12 20 22 4 22 4 12"/><rect x="2" y="7" width="20" height="5"/><line x1="12" y1="22" x2="12" y2="7"/><path d="M12 7H7.5a2.5 2.5 0 0 1 0-5C11 2 12 7 12 7z"/><path d="M12 7h4.5a2.5 2.5 0 0 0 0-5C13 2 12 7 12 7z"/></>,
    share: <><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></>,
    link: <><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></>,
    trash: <><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></>,
    msg: <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>,
    star: <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>,
  };
  return <svg width={s} height={s} viewBox="0 0 24 24" fill="none" stroke={c} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">{p[n]}</svg>;
};

const Orbs = () => (
  <div style={{ position: "absolute", inset: 0, overflow: "hidden", pointerEvents: "none" }}>
    <div style={{ position: "absolute", width: 220, height: 220, borderRadius: "50%", background: "rgba(255,255,255,0.05)", top: -70, right: -50 }}/>
    <div style={{ position: "absolute", width: 140, height: 140, borderRadius: "50%", background: "rgba(255,255,255,0.04)", top: 60, left: -40 }}/>
    <div style={{ position: "absolute", width: 80, height: 80, borderRadius: "50%", background: "rgba(255,255,255,0.045)", top: 30, right: 40 }}/>
    {[{t:100,l:50,s:6},{t:55,l:260,s:4},{t:130,l:140,s:5},{t:170,l:60,s:3}].map((d,i)=>
      <div key={i} style={{ position:"absolute", width:d.s, height:d.s, borderRadius:"50%", background:`rgba(255,255,255,${0.2+i*0.05})`, top:d.t, left:d.l }}/>
    )}
  </div>
);

const Phone = ({ children }) => (
  <div style={{ width:375, minHeight:812, maxHeight:812, background:T.bg, borderRadius:44, border:"3px solid #222", position:"relative", overflow:"hidden", boxShadow:"0 25px 80px rgba(59,47,224,0.4), 0 0 0 1px rgba(255,255,255,0.08)", display:"flex", flexDirection:"column", fontFamily:"-apple-system,BlinkMacSystemFont,'SF Pro Display',sans-serif" }}>
    <div style={{ position:"absolute", top:0, left:"50%", transform:"translateX(-50%)", width:160, height:34, background:"#000", borderRadius:"0 0 20px 20px", zIndex:100 }}/>
    <div style={{ height:54, flexShrink:0, display:"flex", alignItems:"flex-end", justifyContent:"space-between", padding:"0 28px 4px", color:"#fff", fontSize:14, fontWeight:600, zIndex:50 }}>
      <span>9:41</span>
      <span style={{ opacity:0.9, fontSize:12 }}>●●●● ▮</span>
    </div>
    <div style={{ flex:1, overflow:"hidden", display:"flex", flexDirection:"column", position:"relative" }}>{children}</div>
    <div style={{ height:34, flexShrink:0, display:"flex", alignItems:"center", justifyContent:"center" }}>
      <div style={{ width:134, height:5, borderRadius:3, background:"rgba(255,255,255,0.2)" }}/>
    </div>
  </div>
);

const Crd = ({ children, style: s }) => (
  <div style={{ background:T.card, borderRadius:"30px 30px 0 0", flex:1, padding:"26px 22px 16px", display:"flex", flexDirection:"column", boxShadow:"0 -4px 30px rgba(0,0,0,0.08)", ...s }}>{children}</div>
);

const inp = { width:"100%", padding:"15px 16px", borderRadius:12, border:`1px solid ${T.border}`, background:T.cardSurface, fontSize:15, color:T.text, outline:"none", boxSizing:"border-box", marginBottom:10 };

const TabBar = ({ active, onTab }) => (
  <div style={{ display:"flex", justifyContent:"space-around", alignItems:"center", height:52, flexShrink:0, background:T.card, borderTop:`1px solid ${T.border}` }}>
    {[{id:"home",icon:"heart",l:"Envies"},{id:"circles",icon:"users",l:"Cercles"},{id:"community",icon:"hand",l:"Entraide"},{id:"profile",icon:"user",l:"Profil"}].map(t=>
      <button key={t.id} onClick={()=>onTab(t.id)} style={{ display:"flex", flexDirection:"column", alignItems:"center", gap:3, background:"none", border:"none", cursor:"pointer", padding:"4px 16px" }}>
        <I n={t.icon} s={21} c={active===t.id?T.bg:T.textMuted}/><span style={{ fontSize:10, fontWeight:600, color:active===t.id?T.bg:T.textMuted }}>{t.l}</span>
      </button>
    )}
  </div>
);

const Onboarding = ({ step, onNext, onSkip }) => {
  const S = [{e:"🎁",t:"Tes envies,\nau même endroit",s:"Note tout ce qui te ferait plaisir.\nOn s'occupe du reste."},{e:"👥",t:"Partage avec\ntes proches",s:"Ta liste, partagée en 2 taps.\nIls sauront exactement quoi t'offrir."},{e:"🤲",t:"Offre à ceux\nqui en ont besoin",s:"Un geste simple peut tout changer.\nDécouvre le mur d'entraide."}][step];
  return (
    <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
      <Orbs/>
      <button onClick={onSkip} style={{ position:"absolute", top:4, right:20, zIndex:10, background:"rgba(255,255,255,0.15)", backdropFilter:"blur(10px)", border:"none", borderRadius:20, padding:"7px 18px", color:"rgba(255,255,255,0.8)", fontSize:13, fontWeight:500, cursor:"pointer" }}>Passer</button>
      <div style={{ flex:1, display:"flex", alignItems:"center", justifyContent:"center" }}>
        <div style={{ width:110, height:110, borderRadius:30, background:"rgba(255,255,255,0.1)", backdropFilter:"blur(20px)", border:"1px solid rgba(255,255,255,0.12)", display:"flex", alignItems:"center", justifyContent:"center", fontSize:52, boxShadow:"0 12px 40px rgba(0,0,0,0.15)" }}>{S.e}</div>
      </div>
      <Crd style={{ flex:"none", alignItems:"center", paddingBottom:12 }}>
        <div style={{ display:"flex", gap:6, marginBottom:22 }}>
          {[0,1,2].map(i=><div key={i} style={{ width:i===step?28:8, height:8, borderRadius:4, background:i===step?T.bg:"#E0E0E0", transition:"all 0.3s" }}/>)}
        </div>
        <h1 style={{ fontSize:28, fontWeight:800, textAlign:"center", color:T.text, lineHeight:1.15, margin:"0 0 10px", whiteSpace:"pre-line", letterSpacing:-0.5 }}>{S.t}</h1>
        <p style={{ fontSize:15, color:T.textSec, textAlign:"center", lineHeight:1.5, margin:"0 0 28px", whiteSpace:"pre-line" }}>{S.s}</p>
        <button onClick={onNext} style={{ width:"100%", padding:"16px 0", borderRadius:14, background:T.text, color:"#fff", border:"none", fontSize:16, fontWeight:700, cursor:"pointer", boxShadow:"0 4px 15px rgba(0,0,0,0.12)" }}>{step===2?"Commencer":"Continuer"}</button>
        {step===2&&<button onClick={onSkip} style={{ background:"none", border:"none", color:T.bg, fontSize:14, fontWeight:600, marginTop:16, cursor:"pointer" }}>Déjà un compte ? Se connecter</button>}
      </Crd>
    </div>
  );
};

const Auth = ({ mode, onSubmit, onSwitch }) => (
  <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
    <Orbs/>
    <div style={{ flex:"0 0 200", display:"flex", flexDirection:"column", alignItems:"center", justifyContent:"flex-end", paddingBottom:28, position:"relative", zIndex:1 }}>
      <div style={{ width:72, height:72, borderRadius:20, background:"rgba(0,0,0,0.25)", backdropFilter:"blur(20px)", border:"1px solid rgba(255,255,255,0.1)", display:"flex", alignItems:"center", justifyContent:"center", marginBottom:14, boxShadow:"0 8px 32px rgba(0,0,0,0.2)" }}>
        <I n="gift" s={30} c="rgba(255,255,255,0.9)"/>
      </div>
      <span style={{ color:"#fff", fontSize:22, fontWeight:700, letterSpacing:1.5 }}>Offrii</span>
      <div style={{ position:"absolute", left:45, top:70, width:36, height:36, borderRadius:"50%", border:"1px solid rgba(255,255,255,0.08)" }}/>
      <div style={{ position:"absolute", right:55, top:55, width:24, height:24, borderRadius:"50%", border:"1px solid rgba(255,255,255,0.06)" }}/>
    </div>
    <Crd>
      <div style={{ display:"flex", alignItems:"center", gap:8, marginBottom:20 }}>
        <span style={{ fontSize:18 }}>✦</span>
        <h2 style={{ fontSize:24, fontWeight:800, color:T.text, margin:0 }}>{mode==="register"?"Créer un compte":"Se connecter"}</h2>
      </div>
      {mode==="register"&&<input placeholder="Nom d'affichage" style={inp}/>}
      <input placeholder="Email" style={inp}/>
      <input placeholder="Mot de passe" type="password" style={inp}/>
      <button onClick={onSubmit} style={{ width:"100%", padding:"16px 0", borderRadius:14, background:T.text, color:"#fff", border:"none", fontSize:16, fontWeight:700, cursor:"pointer", marginTop:6, boxShadow:"0 4px 15px rgba(0,0,0,0.12)" }}>{mode==="register"?"Commencer":"Se connecter"}</button>
      {mode==="login"&&<button style={{ background:"none", border:"none", color:T.bg, fontSize:14, fontWeight:600, marginTop:14, cursor:"pointer" }}>Mot de passe oublié ?</button>}
      <div style={{ marginTop:"auto", paddingTop:16, textAlign:"center" }}>
        <button onClick={onSwitch} style={{ background:"none", border:"none", color:T.textSec, fontSize:14, cursor:"pointer" }}>
          {mode==="register"?"Déjà un compte ? ":"Pas de compte ? "}
          <span style={{ color:T.text, fontWeight:700, textDecoration:"underline" }}>{mode==="register"?"Login":"Créer un compte"}</span>
        </button>
      </div>
    </Crd>
  </div>
);

const Home = ({ onItem }) => {
  const [cat,setCat]=useState("all");
  const [seg,setSeg]=useState("active");
  const cats=[{id:"all",l:"Toutes"},{id:"tech",l:"Tech"},{id:"mode",l:"Mode"},{id:"maison",l:"Maison"},{id:"loisirs",l:"Loisirs"},{id:"sante",l:"Santé"}];
  const items=[{id:1,name:"AirPods Pro 3",cat:"Tech",price:"279€",prio:3,claimed:false},{id:2,name:"Nike Air Max 90",cat:"Mode",price:"140€",prio:2,claimed:true},{id:3,name:"Lampe bureau LED",cat:"Maison",price:"45€",prio:1,claimed:false},{id:4,name:"Kindle Paperwhite",cat:"Tech",price:"149€",prio:3,claimed:false},{id:5,name:"Coffret thé Mariage Frères",cat:"Maison",price:"35€",prio:2,claimed:false}];
  const pl={1:"Bof",2:"Envie",3:"Très envie"};
  const pc={1:T.textMuted,2:T.accent,3:"#EF4444"};
  return (
    <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
      <Orbs/>
      <div style={{ padding:"4px 22px 16px", position:"relative", zIndex:1 }}>
        <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:14 }}>
          <h1 style={{ fontSize:30, fontWeight:800, color:"#fff", margin:0, letterSpacing:-0.5 }}>Mes Envies</h1>
          <div style={{ width:38, height:38, borderRadius:14, background:"rgba(255,255,255,0.15)", display:"flex", alignItems:"center", justifyContent:"center", color:"#fff", fontSize:15, fontWeight:700, border:"1px solid rgba(255,255,255,0.1)" }}>Y</div>
        </div>
        <div style={{ display:"flex", gap:8, overflowX:"auto" }}>
          {cats.map(c=><button key={c.id} onClick={()=>setCat(c.id)} style={{ padding:"7px 16px", borderRadius:20, border:"none", flexShrink:0, background:cat===c.id?"#fff":"rgba(255,255,255,0.12)", color:cat===c.id?T.bg:"rgba(255,255,255,0.7)", fontSize:13, fontWeight:600, cursor:"pointer" }}>{c.l}</button>)}
        </div>
      </div>
      <Crd style={{ paddingTop:18, paddingBottom:0 }}>
        <div style={{ display:"flex", padding:3, borderRadius:12, background:T.cardSurface, marginBottom:12 }}>
          {["active","purchased"].map(s=><button key={s} onClick={()=>setSeg(s)} style={{ flex:1, padding:"9px 0", borderRadius:10, border:"none", background:seg===s?"#fff":"transparent", color:seg===s?T.text:T.textMuted, fontSize:13, fontWeight:700, cursor:"pointer", boxShadow:seg===s?"0 1px 4px rgba(0,0,0,0.08)":"none" }}>{s==="active"?"En cours":"Offerts"}</button>)}
        </div>
        <div style={{ flex:1, overflow:"auto" }}>
          {items.map(it=>(
            <button key={it.id} onClick={()=>onItem(it)} style={{ width:"100%", display:"flex", alignItems:"center", gap:14, padding:"14px 0", borderBottom:`1px solid ${T.border}`, background:"none", border:"none", borderBottomWidth:1, borderBottomStyle:"solid", borderBottomColor:T.border, cursor:"pointer", textAlign:"left" }}>
              <div style={{ width:46, height:46, borderRadius:14, background:`${T.bg}10`, display:"flex", alignItems:"center", justifyContent:"center", flexShrink:0 }}><I n="gift" s={20} c={T.bg}/></div>
              <div style={{ flex:1, minWidth:0 }}>
                <div style={{ fontSize:15, fontWeight:700, color:T.text, whiteSpace:"nowrap", overflow:"hidden", textOverflow:"ellipsis" }}>{it.name}</div>
                <div style={{ display:"flex", alignItems:"center", gap:8, marginTop:3 }}>
                  <span style={{ fontSize:11, fontWeight:600, padding:"2px 8px", borderRadius:6, background:`${T.bg}10`, color:T.bg }}>{it.cat}</span>
                  <span style={{ fontSize:11, fontWeight:700, color:pc[it.prio] }}>{pl[it.prio]}</span>
                </div>
              </div>
              <div style={{ textAlign:"right", flexShrink:0 }}>
                <div style={{ fontSize:15, fontWeight:800, color:T.text }}>{it.price}</div>
                {it.claimed&&<span style={{ fontSize:10, fontWeight:700, color:T.accent, background:T.accentLight, padding:"2px 7px", borderRadius:6 }}>Réservé</span>}
              </div>
            </button>
          ))}
        </div>
      </Crd>
      <div style={{ position:"absolute", bottom:68, right:22, zIndex:10 }}>
        <button style={{ width:56, height:56, borderRadius:18, background:T.bg, border:"none", boxShadow:"0 8px 28px rgba(59,47,224,0.5)", display:"flex", alignItems:"center", justifyContent:"center", cursor:"pointer" }}><I n="plus" s={26} c="#fff"/></button>
      </div>
    </div>
  );
};

const ItemDetail = ({ item, onBack }) => (
  <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
    <Orbs/>
    <div style={{ padding:"4px 16px 12px", zIndex:1 }}>
      <button onClick={onBack} style={{ background:"rgba(255,255,255,0.15)", border:"none", borderRadius:12, padding:"8px 14px", cursor:"pointer", display:"flex", alignItems:"center", gap:4 }}>
        <I n="chevL" s={18} c="#fff"/><span style={{ color:"#fff", fontSize:14, fontWeight:600 }}>Retour</span>
      </button>
    </div>
    <div style={{ display:"flex", flexDirection:"column", alignItems:"center", padding:"0 0 20px", zIndex:1 }}>
      <div style={{ width:76, height:76, borderRadius:22, background:"rgba(255,255,255,0.1)", border:"1px solid rgba(255,255,255,0.08)", display:"flex", alignItems:"center", justifyContent:"center", marginBottom:12 }}><I n="gift" s={34} c="rgba(255,255,255,0.8)"/></div>
      <h1 style={{ fontSize:22, fontWeight:800, color:"#fff", margin:0, textAlign:"center" }}>{item?.name||"AirPods Pro 3"}</h1>
      <div style={{ display:"flex", gap:8, marginTop:10 }}>
        <span style={{ fontSize:12, fontWeight:700, padding:"4px 12px", borderRadius:10, background:"rgba(255,255,255,0.15)", color:"#fff" }}>{item?.cat||"Tech"}</span>
        <span style={{ fontSize:12, fontWeight:700, padding:"4px 12px", borderRadius:10, background:"rgba(245,158,11,0.25)", color:T.accent }}>Très envie</span>
      </div>
    </div>
    <Crd style={{ paddingTop:22 }}>
      {[{l:"Prix estimé",v:item?.price||"279€"},{l:"Lien",v:"amazon.fr/airpods-pro..."},{l:"Description",v:"La dernière version avec réduction de bruit active et USB-C."}].map((f,i)=>(
        <div key={i} style={{ padding:"14px 16px", marginBottom:8, borderRadius:14, background:T.cardSurface }}>
          <div style={{ fontSize:11, fontWeight:700, color:T.textMuted, marginBottom:4, textTransform:"uppercase", letterSpacing:0.5 }}>{f.l}</div>
          <div style={{ fontSize:15, color:T.text, fontWeight:500 }}>{f.v}</div>
        </div>
      ))}
      {item?.claimed&&<div style={{ padding:"14px 16px", borderRadius:14, background:T.accentLight, display:"flex", alignItems:"center", gap:10, marginTop:4 }}><span>✨</span><span style={{ fontSize:14, fontWeight:700, color:T.accent }}>Quelqu'un s'en occupe !</span></div>}
      <div style={{ display:"flex", gap:10, marginTop:"auto", paddingTop:16 }}>
        <button style={{ flex:1, padding:"15px 0", borderRadius:14, background:T.success, color:"#fff", border:"none", fontSize:15, fontWeight:700, cursor:"pointer" }}>Marquer reçu</button>
        <button style={{ padding:"15px", borderRadius:14, background:T.cardSurface, border:"none", cursor:"pointer" }}><I n="share" s={18} c={T.textSec}/></button>
        <button style={{ padding:"15px", borderRadius:14, background:"#FEE2E2", border:"none", cursor:"pointer" }}><I n="trash" s={18} c={T.danger}/></button>
      </div>
    </Crd>
  </div>
);

const Circles = ({ onCircle }) => {
  const circles=[{id:1,name:"Famille",members:["M","P","S"],items:12,direct:false},{id:2,name:"Avec Sophie",members:["S"],items:8,direct:true},{id:3,name:"Amis proches",members:["A","M","K","L"],items:23,direct:false}];
  const mc=[T.bg,T.accent,T.success,"#8B5CF6"];
  return (
    <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
      <Orbs/>
      <div style={{ padding:"4px 22px 16px", zIndex:1, display:"flex", justifyContent:"space-between", alignItems:"center" }}>
        <h1 style={{ fontSize:30, fontWeight:800, color:"#fff", margin:0 }}>Cercles</h1>
        <button style={{ width:38, height:38, borderRadius:14, background:"rgba(255,255,255,0.15)", border:"1px solid rgba(255,255,255,0.1)", display:"flex", alignItems:"center", justifyContent:"center", cursor:"pointer" }}><I n="plus" s={18} c="#fff"/></button>
      </div>
      <Crd style={{ paddingTop:18 }}>
        <div style={{ flex:1, overflow:"auto" }}>
          {circles.map(c=>(
            <button key={c.id} onClick={()=>onCircle(c)} style={{ width:"100%", padding:"18px", marginBottom:10, borderRadius:18, background:T.cardSurface, border:"none", cursor:"pointer", textAlign:"left" }}>
              <div style={{ display:"flex", justifyContent:"space-between", alignItems:"flex-start" }}>
                <div>
                  <div style={{ fontSize:17, fontWeight:800, color:T.text }}>{c.direct?"💬 ":""}{c.name}</div>
                  <div style={{ fontSize:12, color:T.textMuted, marginTop:3 }}>{c.items} envies • {c.members.length+1} membres</div>
                </div>
                <I n="chevR" s={16} c={T.textMuted}/>
              </div>
              <div style={{ display:"flex", marginTop:12 }}>
                {c.members.map((m,i)=><div key={i} style={{ width:30, height:30, borderRadius:10, background:mc[i%4], display:"flex", alignItems:"center", justifyContent:"center", color:"#fff", fontSize:11, fontWeight:800, border:"2px solid #fff", marginLeft:i>0?-6:0, zIndex:10-i }}>{m}</div>)}
              </div>
            </button>
          ))}
          <div style={{ display:"flex", gap:10, marginTop:10 }}>
            <button style={{ flex:1, padding:"14px", borderRadius:14, border:`2px dashed ${T.bg}40`, background:`${T.bg}06`, color:T.bg, fontSize:13, fontWeight:700, cursor:"pointer" }}>+ Créer un cercle</button>
            <button style={{ flex:1, padding:"14px", borderRadius:14, border:`2px dashed ${T.accent}40`, background:`${T.accent}06`, color:T.accent, fontSize:13, fontWeight:700, cursor:"pointer" }}>↗ Partager ma liste</button>
          </div>
        </div>
      </Crd>
    </div>
  );
};

const CircleDetail = ({ circle, onBack }) => {
  const members=[{name:"Toi",items:[{name:"AirPods Pro 3",price:"279€",claimed:true},{name:"Kindle Paperwhite",price:"149€",claimed:false}]},{name:circle?.direct?"Sophie":"Maman",items:[{name:"Écharpe cachemire",price:"85€",claimed:false},{name:"Livre de cuisine",price:"25€",claimed:false}]}];
  return (
    <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
      <Orbs/>
      <div style={{ padding:"4px 16px 14px", zIndex:1, display:"flex", alignItems:"center", gap:12 }}>
        <button onClick={onBack} style={{ background:"rgba(255,255,255,0.15)", border:"none", borderRadius:12, padding:"8px 12px", cursor:"pointer" }}><I n="chevL" s={18} c="#fff"/></button>
        <h2 style={{ fontSize:20, fontWeight:800, color:"#fff", margin:0, flex:1 }}>{circle?.name||"Famille"}</h2>
        <button style={{ background:"rgba(255,255,255,0.15)", border:"none", borderRadius:12, padding:"8px 12px", cursor:"pointer" }}><I n="share" s={16} c="#fff"/></button>
      </div>
      <Crd style={{ paddingTop:16 }}>
        <button style={{ width:"100%", padding:"12px", borderRadius:14, background:`${T.bg}08`, border:`1.5px solid ${T.bg}25`, color:T.bg, fontSize:14, fontWeight:700, cursor:"pointer", display:"flex", alignItems:"center", justifyContent:"center", gap:8, marginBottom:14 }}><I n="plus" s={16} c={T.bg}/> Inviter quelqu'un</button>
        <div style={{ flex:1, overflow:"auto" }}>
          {members.map((m,mi)=>(
            <div key={mi} style={{ marginBottom:16 }}>
              <div style={{ display:"flex", alignItems:"center", gap:10, marginBottom:8 }}>
                <div style={{ width:28, height:28, borderRadius:10, background:mi===0?T.bg:T.accent, display:"flex", alignItems:"center", justifyContent:"center", color:"#fff", fontSize:11, fontWeight:800 }}>{m.name[0]}</div>
                <span style={{ fontSize:15, fontWeight:800, color:T.text }}>{m.name}</span>
              </div>
              {m.items.map((it,ii)=>(
                <div key={ii} style={{ padding:"14px 16px", marginBottom:6, borderRadius:14, background:T.cardSurface, display:"flex", alignItems:"center", justifyContent:"space-between" }}>
                  <div><div style={{ fontSize:14, fontWeight:700, color:T.text }}>{it.name}</div><div style={{ fontSize:12, color:T.textMuted, marginTop:2 }}>{it.price}</div></div>
                  {mi===0?(it.claimed?<span style={{ fontSize:10, fontWeight:800, color:T.accent, background:T.accentLight, padding:"4px 10px", borderRadius:8 }}>Réservé ✨</span>:null):(
                    <button style={{ padding:"8px 14px", borderRadius:10, background:T.accent, color:"#fff", border:"none", fontSize:12, fontWeight:800, cursor:"pointer" }}>Je m'en occupe</button>
                  )}
                </div>
              ))}
            </div>
          ))}
        </div>
      </Crd>
    </div>
  );
};

const Community = () => {
  const [cat,setCat]=useState("Toutes");
  const cats=["Toutes","Éducation","Vêtements","Santé","Enfants","Religion"];
  const wishes=[{title:"Un manteau d'hiver pour ma fille",cat:"Vêtements",author:"Fatima",desc:"Ma fille de 7 ans n'a pas de manteau chaud. Taille 8 ans.",status:"open"},{title:"Un Coran en français",cat:"Religion",author:"Ibrahim",desc:"Je viens de me convertir et j'aimerais lire le Coran.",status:"open"},{title:"Des livres pour apprendre à lire",cat:"Éducation",author:"Marie",desc:"Mon fils entre en CP, je cherche des livres adaptés.",status:"matched"},{title:"Un cartable neuf",cat:"Éducation",author:"Ahmed",desc:"Pour la rentrée de mon fils. L'ancien est usé.",status:"fulfilled"}];
  const cc={"Vêtements":"#8B5CF6","Religion":"#6366F1","Éducation":"#3B82F6","Enfants":"#EC4899","Santé":T.success};
  return (
    <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
      <Orbs/>
      <div style={{ padding:"4px 22px 16px", zIndex:1 }}>
        <h1 style={{ fontSize:30, fontWeight:800, color:"#fff", margin:"0 0 4px" }}>Entraide</h1>
        <p style={{ fontSize:13, color:"rgba(255,255,255,0.55)", margin:"0 0 14px" }}>Des gestes simples, des impacts réels</p>
        <div style={{ display:"flex", gap:7, overflowX:"auto" }}>
          {cats.map(c=><button key={c} onClick={()=>setCat(c)} style={{ padding:"7px 14px", borderRadius:20, border:"none", flexShrink:0, background:cat===c?"#fff":"rgba(255,255,255,0.12)", color:cat===c?T.bg:"rgba(255,255,255,0.7)", fontSize:12, fontWeight:600, cursor:"pointer" }}>{c}</button>)}
        </div>
      </div>
      <Crd style={{ paddingTop:16, paddingBottom:0 }}>
        <div style={{ flex:1, overflow:"auto" }}>
          {wishes.map((w,i)=>(
            <div key={i} style={{ padding:"16px", marginBottom:10, borderRadius:16, background:T.cardSurface, border:w.status==="fulfilled"?`2px solid ${T.success}`:"none" }}>
              <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:8 }}>
                <span style={{ fontSize:11, fontWeight:800, padding:"3px 10px", borderRadius:8, background:`${cc[w.cat]||T.bg}15`, color:cc[w.cat]||T.bg }}>{w.cat}</span>
                {w.status==="fulfilled"&&<span style={{ fontSize:11, fontWeight:800, color:T.success }}>Offert ! 🎉</span>}
                {w.status==="matched"&&<span style={{ fontSize:11, fontWeight:700, color:T.accent }}>En cours...</span>}
              </div>
              <h3 style={{ fontSize:15, fontWeight:800, color:T.text, margin:"0 0 4px" }}>{w.title}</h3>
              <p style={{ fontSize:13, color:T.textSec, margin:"0 0 12px", lineHeight:1.4 }}>{w.desc}</p>
              <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center" }}>
                <span style={{ fontSize:12, color:T.textMuted, fontWeight:600 }}>{w.author}</span>
                {w.status==="open"&&<button style={{ padding:"8px 18px", borderRadius:10, background:T.accent, color:"#fff", border:"none", fontSize:12, fontWeight:800, cursor:"pointer", boxShadow:"0 4px 12px rgba(245,158,11,0.25)" }}>Je veux offrir</button>}
              </div>
            </div>
          ))}
        </div>
        <div style={{ padding:"8px 0 4px", flexShrink:0 }}>
          <button style={{ width:"100%", padding:"14px 0", borderRadius:14, background:`${T.bg}08`, border:`1.5px solid ${T.bg}20`, color:T.bg, fontSize:14, fontWeight:700, cursor:"pointer" }}>Publier un souhait</button>
        </div>
      </Crd>
    </div>
  );
};

const Profile = () => (
  <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
    <Orbs/>
    <div style={{ padding:"4px 22px 22px", zIndex:1, display:"flex", flexDirection:"column", alignItems:"center" }}>
      <div style={{ width:72, height:72, borderRadius:22, background:"rgba(255,255,255,0.12)", border:"2px solid rgba(255,255,255,0.15)", display:"flex", alignItems:"center", justifyContent:"center", color:"#fff", fontSize:28, fontWeight:800, marginBottom:10 }}>Y</div>
      <div style={{ fontSize:20, fontWeight:800, color:"#fff" }}>Yassine</div>
      <div style={{ fontSize:13, color:"rgba(255,255,255,0.55)", marginTop:2 }}>yassine@offrii.com</div>
    </div>
    <Crd style={{ paddingTop:22 }}>
      <div style={{ flex:1, overflow:"auto" }}>
        {[{t:"RAPPELS",items:[{l:"Fréquence",v:"Hebdomadaire"},{l:"Heure",v:"10:00"}]},{t:"PARTAGES",items:[{l:"Liens actifs",v:"2 liens"}]},{t:"DONNÉES",items:[{l:"Exporter mes données",v:""},{l:"Supprimer mon compte",v:"",d:true}]}].map((sec,si)=>(
          <div key={si} style={{ marginBottom:18 }}>
            <div style={{ fontSize:11, fontWeight:800, color:T.textMuted, letterSpacing:1.5, marginBottom:8 }}>{sec.t}</div>
            <div style={{ borderRadius:16, overflow:"hidden", background:T.cardSurface }}>
              {sec.items.map((it,ii)=>(
                <div key={ii} style={{ padding:"14px 16px", display:"flex", justifyContent:"space-between", alignItems:"center", borderBottom:ii<sec.items.length-1?`1px solid ${T.border}`:"none" }}>
                  <span style={{ fontSize:15, fontWeight:600, color:it.d?T.danger:T.text }}>{it.l}</span>
                  <div style={{ display:"flex", alignItems:"center", gap:6 }}>
                    {it.v&&<span style={{ fontSize:14, color:T.textMuted }}>{it.v}</span>}
                    <I n="chevR" s={14} c={T.textMuted}/>
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
        <div style={{ textAlign:"center", padding:16, fontSize:12, color:T.textMuted }}>Offrii v1.0</div>
      </div>
    </Crd>
  </div>
);

export default function OffriiApp() {
  const [screen,setScreen]=useState("onboarding");
  const [obStep,setObStep]=useState(0);
  const [authMode,setAuthMode]=useState("register");
  const [tab,setTab]=useState("home");
  const [selItem,setSelItem]=useState(null);
  const [selCircle,setSelCircle]=useState(null);

  const showTabs=["home","circles","community","profile"].includes(screen);
  const nav=(s,t)=>{setScreen(s);if(t)setTab(t);setSelItem(null);setSelCircle(null);};

  return (
    <div style={{ minHeight:"100vh", display:"flex", flexDirection:"column", alignItems:"center", justifyContent:"center", background:"#080810", padding:"20px 10px", fontFamily:"-apple-system,BlinkMacSystemFont,'SF Pro Display',sans-serif" }}>
      <div style={{ display:"flex", gap:6, marginBottom:20, flexWrap:"wrap", justifyContent:"center", maxWidth:420 }}>
        {[{l:"Onboarding",s:"onboarding"},{l:"Inscription",s:"auth-r"},{l:"Connexion",s:"auth-l"},{l:"Envies",s:"home"},{l:"Cercles",s:"circles"},{l:"Entraide",s:"community"},{l:"Profil",s:"profile"}].map(b=>{
          const a=screen===b.s||(b.s==="auth-r"&&screen==="auth"&&authMode==="register")||(b.s==="auth-l"&&screen==="auth"&&authMode==="login");
          return <button key={b.s} onClick={()=>{
            if(b.s==="auth-r"){setScreen("auth");setAuthMode("register");}
            else if(b.s==="auth-l"){setScreen("auth");setAuthMode("login");}
            else nav(b.s,["home","circles","community","profile"].includes(b.s)?b.s:tab);
            setObStep(0);
          }} style={{ padding:"6px 12px", borderRadius:8, border:"none", fontSize:11, fontWeight:700, background:a?T.bg:"#1a1a2e", color:a?"#fff":"#555", cursor:"pointer" }}>{b.l}</button>;
        })}
      </div>
      <Phone>
        <div style={{ flex:1, display:"flex", flexDirection:"column", position:"relative" }}>
          {screen==="onboarding"&&<Onboarding step={obStep} onNext={()=>obStep<2?setObStep(obStep+1):(setScreen("auth"),setAuthMode("register"))} onSkip={()=>{setScreen("auth");setAuthMode("register");}}/>}
          {screen==="auth"&&<Auth mode={authMode} onSubmit={()=>nav("home","home")} onSwitch={()=>setAuthMode(authMode==="register"?"login":"register")}/>}
          {screen==="home"&&<Home onItem={it=>{setSelItem(it);setScreen("itemDetail");}}/>}
          {screen==="itemDetail"&&<ItemDetail item={selItem} onBack={()=>setScreen("home")}/>}
          {screen==="circles"&&<Circles onCircle={c=>{setSelCircle(c);setScreen("circleDetail");}}/>}
          {screen==="circleDetail"&&<CircleDetail circle={selCircle} onBack={()=>setScreen("circles")}/>}
          {screen==="community"&&<Community/>}
          {screen==="profile"&&<Profile/>}
        </div>
        {showTabs&&<TabBar active={tab} onTab={t=>{setTab(t);setScreen(t);}}/>}
      </Phone>
      <p style={{ color:"#444", fontSize:11, marginTop:16 }}>Offrii Prototype — Navigue via les boutons ou clique dans l'app</p>
    </div>
  );
}
