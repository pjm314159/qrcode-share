# AI Image Generation Prompts

This document contains detailed prompts for generating decorative images used in the QRcode Share frontend. All images follow the Clay design system aesthetic: 3D claymation-style illustrations on a cream canvas background (#fffaf0), with warm saturated brand colors (pink #ff4d8b, teal #1a3a3a, lavender #b8a4ed, peach #ffb084, ochre #e8b94a, mint #a4d4c5, coral #ff6b5a).

---

## Active Images

### 1. Hero Illustration - Home Page

**File:** `hero-illustration.png`
**Size:** 800x600px
**Location:** HomePage, hero section (centered below title, with decorative CSS blobs behind)
**Status:** Active (updated prompt for Phase 2 -- transparent background for blob integration)

**Prompt:**

> A 3D claymation-style illustration of a smartphone floating in the center, emitting a glowing QR code from its screen. The QR code is dissolving into colorful particles in hot pink (#ff4d8b), lavender (#b8a4ed), and peach (#ffb084) that flow outward and transform into small link icons and share symbols. The phone has a warm cream-white body with soft rounded edges. In the background, abstract 3D clay shapes - small mountains in ochre (#e8b94a) and teal (#1a3a3a), floating dots in mint (#a4d4c5), and curved ribbons in coral (#ff6b5a). The entire scene has a soft, hand-crafted, playful aesthetic like a stop-motion animation set. IMPORTANT: The background must be fully transparent (PNG with alpha channel) -- the illustration will be placed on a gradient background with CSS decorative blobs behind it, so the illustration must blend seamlessly. The outer edges of the particles and shapes should fade softly into transparency. No solid background color. No text. No people. Clean, modern, friendly. Soft studio lighting with gentle shadows. The style should feel like premium 3D rendered product photography with a clay/plastic material aesthetic.

**Design integration note:** In Phase 2, this illustration is centered below the title on a warm gradient background (`from-canvas to-brand-peach/10`) with CSS `DecorativeBlob` components (soft blurred circles in brand-lavender, brand-mint, and brand-pink) positioned behind it. The transparent background allows the blobs and gradient to show through, creating a cohesive visual composition. The illustration is given a `drop-shadow-lg` CSS class to lift it off the background.

---

### 2. Logo Mark

**File:** `logo-mark.png`
**Size:** 256x256px (also need 64x64 for favicon)
**Location:** Layout header, favicon
**Status:** Active

**Prompt:**

> A minimal 3D claymation-style logo mark for a QR code sharing app. A rounded square shape in near-black (#0a0a0a) with soft clay-like edges, containing a stylized QR code pattern made of small rounded squares in cream white (#fffaf0). The QR code pattern is simplified and abstract - only 9 squares in a 3x3 grid arrangement, with the center square replaced by a small share/arrow icon in hot pink (#ff4d8b). The entire mark has a soft, tactile, hand-crafted 3D appearance with subtle lighting and gentle shadows. Clean, modern, friendly. No text. The material should look like smooth matte clay or soft plastic. Warm studio lighting.

---

### 3. Empty State - No Messages

**File:** `empty-inbox.png`
**Size:** 320x240px
**Location:** ChannelPage when no messages exist
**Status:** Active

**Prompt:**

> A 3D claymation-style illustration of an empty open box or inbox tray. The box is made of warm cream-colored clay (#f5f0e0) with soft rounded edges. Inside the box is completely empty with a subtle shadow. A small paper airplane in lavender (#b8a4ed) is perched on the edge of the box, as if about to take off. Next to the box, a small QR code fragment in ochre (#e8b94a) lies on the cream background (#fffaf0). The scene conveys a friendly, inviting feeling - "waiting for something to arrive." Soft studio lighting, gentle shadows, hand-crafted clay aesthetic. No text. No people. Warm and approachable.

---

### 4. 404 Not Found

**File:** `not-found.png`
**Size:** 320x240px
**Location:** NotFoundPage
**Status:** Active (size reduced from 400x300 to 320x240 for Phase 2)

**Prompt:**

> A 3D claymation-style illustration of a magnifying glass lying on a cream surface (#fffaf0), tilted at an angle. The magnifying glass has a warm ochre handle (#e8b94a) and a glass lens that reflects a subtle question mark shape. Next to the magnifying glass, small scattered 3D clay puzzle pieces in hot pink (#ff4d8b), teal (#1a3a3a), and lavender (#b8a4ed) are spread across the surface, suggesting something is missing or incomplete. One puzzle piece is slightly elevated as if mid-fall. The scene is playful and curious, not sad or alarming. Soft studio lighting, gentle shadows, hand-crafted clay material aesthetic. No text. No people.

---

### 5. Scan Feature Illustration

**File:** `feature-scan.png`
**Size:** 320x240px
**Location:** HomePage, scan feature card
**Status:** Active (size reduced from 400x300 to 320x240 for Phase 2)

**Prompt:**

> A 3D claymation-style close-up illustration of a smartphone camera viewfinder. The phone is in a hot pink (#ff4d8b) clay case with rounded edges. On the phone screen, a simplified QR code is being scanned - shown as a glowing frame with corner brackets in white. Small colorful light particles in peach (#ffb084) and mint (#a4d4c5) float upward from the QR code, suggesting data being captured. The background is solid hot pink (#ff4d8b) - this image will be placed on a pink feature card. The phone and elements should feel tactile, like clay or soft plastic. No text. No people. Clean, modern, playful. Soft studio lighting.

---

### 6. Share Feature Illustration

**File:** `feature-share.png`
**Size:** 320x240px
**Location:** HomePage, share feature card
**Status:** Active (size reduced from 400x300 to 320x240 for Phase 2)

**Prompt:**

> A 3D claymation-style illustration of a broadcast/antenna tower made of clay in deep teal (#1a3a3a). The tower emits concentric circular signal waves in white and light teal, spreading outward. Small link icons (simplified chain links) in cream white (#fffaf0) float in the signal waves, being broadcast outward. At the base of the tower, small clay devices (a phone, a tablet) in ochre (#e8b94a) and lavender (#b8a4ed) receive the signals. The background is solid deep teal (#1a3a3a) - this image will be placed on a teal feature card. Hand-crafted clay aesthetic with soft shadows. No text. No people. Clean, modern, friendly.

---

### 7. Open Feature Illustration

**File:** `feature-open.png`
**Size:** 320x240px
**Location:** HomePage, open feature card
**Status:** Active (size reduced from 400x300 to 320x240 for Phase 2)

**Prompt:**

> A 3D claymation-style illustration of a web browser window made of soft lavender clay (#b8a4ed). The browser has rounded tab shapes at the top and a simplified address bar. Inside the browser window, a bright arrow in near-black (#0a0a0a) points outward to the right, suggesting "opening" or "navigating." Small sparkle effects in ochre (#e8b94a) and mint (#a4d4c5) surround the arrow. The browser window has a friendly, hand-crafted appearance with soft edges and gentle shadows. The background is solid lavender (#b8a4ed) - this image will be placed on a lavender feature card. No text. No people. Clean, modern, playful. Soft studio lighting.

---

### 8. Create Channel Illustration

**File:** `feature-create.png`
**Size:** 320x240px
**Location:** HomePage, create channel card
**Status:** Active (size reduced from 400x300 to 320x240 for Phase 2)

**Prompt:**

> A 3D claymation-style illustration of a rocket ship made of warm peach clay (#ffb084) launching upward from a small cream-colored (#f5f0e0) launch pad. The rocket has a simplified, friendly design with a rounded nose cone and small fins. A trail of small stars and sparkles in ochre (#e8b94a) and mint (#a4d4c5) follows behind the rocket. The rocket carries a small QR code symbol on its body in near-black (#0a0a0a). The background is solid warm peach (#ffb084) - this image will be placed on a peach feature card. Hand-crafted clay aesthetic with soft shadows. No text. No people. Clean, modern, friendly. Soft studio lighting.

---

### 9. Password Protection Illustration

**File:** `password-lock.png`
**Size:** 240x180px
**Location:** PasswordModal, ChannelPage password prompt
**Status:** Active

**Prompt:**

> A 3D claymation-style illustration of a padlock made of ochre clay (#e8b94a) with a rounded, friendly design. The padlock is in the locked position with a small keyhole visible. A small key in near-black (#0a0a0a) floats nearby, slightly rotated as if about to be inserted. The lock has a soft, tactile appearance with gentle shadows and warm studio lighting. The background is transparent (PNG with alpha channel). Hand-crafted clay aesthetic. No text. No people. The lock should feel protective but not threatening - friendly security.

---

### 10. Footer Mountain Illustration

**File:** `footer-mountains.png`
**Size:** 1280x200px
**Location:** Layout footer, bottom decorative element
**Status:** Active

**Prompt:**

> A wide panoramic 3D claymation-style illustration of a mountain range silhouette at the bottom of the image. The mountains are made of soft clay in warm tones - the foreground mountains in ochre (#e8b94a), middle mountains in peach (#ffb084), and distant mountains in lavender (#b8a4ed). The mountains have smooth, rounded peaks (not sharp) consistent with a clay aesthetic. Small floating clouds in cream white (#fffaf0) sit between the peaks. The sky above fades from the cream canvas color (#fffaf0) at the top to a slightly warmer tone near the mountains. The entire illustration is a decorative horizon line, meant to sit at the very bottom of a web page footer. Hand-crafted, warm, inviting. No text. No people. Soft lighting. The image should tile seamlessly or be wide enough for a 1280px container.

---

## Deprecated Images

The following images are deprecated in Phase 2. They are replaced by SVG icon components which are more scalable, lighter, and consistent with the design system.

### 11. Connection Status - Connected (DEPRECATED)

**File:** `status-connected.png`
**Size:** 48x48px
**Status:** DEPRECATED -- Replaced by `<IconSuccess size={12} className="text-success" />`

**Reason:** SVG icons provide better scalability and consistency. The connection status indicator is a small 12px dot that benefits from vector rendering rather than a raster image.

---

### 12. Connection Status - Connecting (DEPRECATED)

**File:** `status-connecting.png`
**Size:** 48x48px
**Status:** DEPRECATED -- Replaced by `<IconConnecting size={12} className="text-warning" />`

**Reason:** Same as above. SVG icon with CSS animation provides a better visual effect than a static PNG.

---

### 13. Connection Status - Disconnected (DEPRECATED)

**File:** `status-disconnected.png`
**Size:** 48x48px
**Status:** DEPRECATED -- Replaced by `<IconDisconnected size={12} className="text-error" />`

**Reason:** Same as above. SVG icon is more appropriate for a status indicator at this size.

---

### 14. Favicon (DEPRECATED)

**File:** `2-64x64.png`
**Size:** 64x64px
**Status:** DEPRECATED -- Replaced by SVG favicon or `logo-mark.png` at appropriate size

**Reason:** The logo-mark.png can serve as the favicon when resized. A dedicated 64x64 PNG is no longer needed.

---

## Image Usage Guidelines

### File Format
- All images: PNG with transparency where noted
- Hero illustration: PNG or WebP for performance
- Logo: PNG + SVG fallback
- Status indicators: SVG icons (not PNG images)

### Color Reference
All colors reference the Clay design system from DESIGN.md:
- Canvas: #fffaf0
- Ink: #0a0a0a
- Brand Pink: #ff4d8b
- Brand Teal: #1a3a3a
- Brand Lavender: #b8a4ed
- Brand Peach: #ffb084
- Brand Ochre: #e8b94a
- Brand Mint: #a4d4c5
- Brand Coral: #ff6b5a
- Surface Card: #f5f0e0
- Surface Soft: #faf5e8

### Style Consistency
All images must share:
1. **3D claymation aesthetic** - hand-crafted, tactile, rounded forms
2. **Soft studio lighting** - gentle shadows, no harsh contrasts
3. **Warm cream canvas** - consistent background tone
4. **Saturated brand colors** - from the Clay palette only
5. **No text** - all text is rendered by the web app
6. **No people/characters** - use objects and abstract shapes only
7. **Friendly and approachable** - never threatening or cold

### File Naming Convention
- Use kebab-case: `feature-scan.png`, `status-connected.png`
- Place in `public/images/` directory
- Reference in components as `/images/filename.png`

### Phase 2 Design Integration

In Phase 2, the hero illustration is the primary visual change. It now uses a transparent background to integrate with CSS decorative blobs and gradient backgrounds. This creates a layered visual composition:

1. **Gradient background layer** -- `bg-gradient-to-b from-canvas to-brand-peach/10` on the hero section
2. **Decorative blob layer** -- `DecorativeBlob` components with `blur(60px)` in brand-lavender, brand-mint, and brand-pink at low opacity
3. **Illustration layer** -- `hero-illustration.png` with transparent background and `drop-shadow-lg`

Feature card illustrations and the not-found illustration have been reduced in size to support the more concise page layouts:
- Feature cards: 400x300px -> 320x240px
- Not found: 400x300px -> 320x240px
- If existing images at the larger size already exist, they can be used with CSS sizing (`width={120} height={90}`) rather than regenerating at the smaller size.

No new images are needed. Decorative background elements (blobs) are rendered with CSS, not images.
