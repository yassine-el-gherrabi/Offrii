# iOS App Launch Experience & Onboarding Research Report

**Date**: 2026-03-11
**Scope**: Launch screens, FTUE, onboarding flows, auth placement, permissions, post-auth UX, returning users, startup performance
**Confidence Level**: High (85%) -- based on 40+ sources across Apple documentation, UX research, industry benchmarks, and real app teardowns

---

## Executive Summary

The first 3-7 days after install determine whether a user stays or churns. iOS apps average a 25.6% Day 1 retention rate, meaning roughly 75% of users abandon an app within 24 hours. Apps with structured onboarding campaigns see a 24% engagement score vs 17% for apps without (Airship, Q2 2024). Effective onboarding can increase retention by up to 50% (Localytics). For a community gifting/wishlist app like Offrii, the critical path is: fast launch -> value demonstration -> low-friction auth -> contextual permissions -> immediate first action (browsing local offers or posting a wish).

---

## 1. Launch Screen / Splash Screen

### Apple's Official HIG Position

Apple's Human Interface Guidelines are explicit:

> "The launch screen isn't a branding opportunity. Avoid creating a screen that looks like a splash screen or an 'About' window, and don't include logos or other branding elements."

**Apple's requirements:**
- The launch screen should appear **instantly** when the app opens
- It should be **quickly replaced** by the app's first screen
- It should **resemble the first screen** of the app (same layout structure, minus dynamic content)
- Purpose: decrease the **perception** of long load times, not to brand or market

**What Apple says NOT to do:**
- No logos or branding elements
- No "About" window styling
- No text that requires localization (it is a static asset)
- No advertising or marketing content

### What Top Apps Actually Do (The Reality)

Despite Apple's guidelines, many top apps use a branded launch moment. The key distinction is between:

1. **Static Launch Screen (LaunchScreen.storyboard)** -- Required by Apple, shown by iOS instantly
2. **Animated Splash/Brand Moment** -- A custom SwiftUI/UIKit view shown AFTER the static launch screen, before the main app

**Examples:**
| App | Approach | Duration | Notes |
|-----|----------|----------|-------|
| Netflix | Animated "Tudum" sound + logo | ~2s | Iconic brand moment, skippable |
| Airbnb | Static launch matching first screen | <1s | Follows Apple HIG closely |
| Instagram | Static gradient + logo | <1s | Minimal brand + fast transition |
| OLIO | Logo on branded color | ~1.5s | Simple brand recognition |
| Nextdoor | Green background + logo | ~1s | Quick brand, then neighborhood feed |
| Duolingo | Green + owl mascot | ~1s | Character-based brand recognition |

### Duration Best Practices

- **Target**: Less than 2 seconds total (static + any animated transition)
- **Maximum tolerable**: 3 seconds before users perceive delay
- Larger brands (Netflix, Disney+) stretch to 3-4s with animation, but this is risky for smaller apps
- Apps like (Not Boring) Weather use 8-second animations, but these are skippable and extremely rare

### Recommendation for Offrii

**Use a hybrid approach:**
1. **Static LaunchScreen.storyboard**: App background color with the Offrii logo centered (minimal, follows Apple spirit while establishing brand)
2. **Brief animated transition** (~0.5-1s): The logo subtly animates/fades into the real first screen, making the transition feel seamless
3. **Total perceived launch**: Under 2 seconds
4. The static screen should structurally mirror the first real screen (tab bar outline, background color) so the transition feels instant

---

## 2. First Launch vs Returning User Detection

### Architecture Pattern

The standard iOS pattern for differentiating first-time vs returning users:

```
App Launch
    |
    v
Check auth token in Keychain
    |
    +-- No token found --> Check UserDefaults "hasLaunchedBefore"
    |                          |
    |                          +-- false (first ever launch) --> Onboarding Flow
    |                          +-- true (returning, logged out) --> Login Screen
    |
    +-- Token found --> Validate token
                          |
                          +-- Valid --> Main App (with background data refresh)
                          +-- Expired --> Attempt silent refresh
                                           |
                                           +-- Success --> Main App
                                           +-- Failure --> Login Screen
```

### Implementation Details

**First launch detection:**
- Use `UserDefaults.standard.bool(forKey: "hasCompletedOnboarding")` -- defaults to `false` on fresh install
- Set to `true` after onboarding completion (not on first screen view -- in case they quit mid-flow)
- Important: `UserDefaults` persists through app updates but is cleared on uninstall

**Auth state detection:**
- Store auth tokens in **Keychain** (not UserDefaults) -- persists across reinstalls on same device
- On fresh install, **clear Keychain** of any stale tokens from a previous install (use a UserDefaults flag for this)
- Check token validity on launch before making routing decision

**Key pattern for reinstalls:**
```swift
// In AppDelegate/App init
let isFirstLaunch = !UserDefaults.standard.bool(forKey: "hasEverLaunched")
if isFirstLaunch {
    // Clear any stale Keychain data from previous install
    KeychainService.clearAll()
    UserDefaults.standard.set(true, forKey: "hasEverLaunched")
}
```

### What to Show on Each Path

| Scenario | What to Show |
|----------|-------------|
| First ever launch | Value proposition screens -> Onboarding -> Auth |
| Reinstall (no auth) | Abbreviated welcome -> Auth (skip full onboarding) |
| Returning + valid token | Main app immediately (skeleton -> content) |
| Returning + expired token | Main app with cached data while silently refreshing token |
| Returning + auth failure | Login screen with friendly "Welcome back" messaging |

---

## 3. Onboarding Approaches

### The Three Main Approaches

#### A. Classic Carousel/Walkthrough (Swipe-through screens)
- 3-5 static or animated screens explaining value proposition
- Usually shown before auth
- Skip button available
- **Pros**: Controlled narrative, brand storytelling, sets expectations
- **Cons**: Many users skip, doesn't demonstrate real value, adds friction
- **Retention impact**: Mixed -- depends heavily on execution quality

#### B. Progressive/Contextual Onboarding
- No upfront tutorial; features explained via tooltips, overlays, and coach marks as users encounter them
- User gets into the real app immediately
- **Pros**: Learning happens in context, reduces time-to-value, no skip problem
- **Cons**: Users may miss features, harder to implement, requires more design work
- **Retention impact**: 20-40% increase in onboarding completion rates (GuideJar, 2025)
- **Examples**: Waze (explains traffic reporting only when navigating), Slack (tooltips on first use of each feature)

#### C. Hybrid: Value Proposition + Progressive
- 2-3 brief value screens (not feature tutorials) followed by progressive in-app education
- The upfront screens answer "What is this app?" not "How do I use this app?"
- **Pros**: Combines brand storytelling with contextual learning
- **Cons**: Requires careful balance
- **This is the current industry best practice for 2024-2025**

### Retention Statistics

| Metric | Value | Source |
|--------|-------|--------|
| Average iOS Day 1 retention | 25.6-27% | Adjust 2024, multiple sources |
| Average iOS Day 7 retention | 13-14% | Adjust 2024 |
| Average iOS Day 30 retention | 7-8% | Adjust 2024 |
| Social app Day 1 retention (elite) | >60% | AppSamurai 2025 |
| Social app Day 1 retention (average) | 35-45% | AppSamurai 2025 |
| Marketplace Day 1 retention | 25% | Mobile App Trends 2025 |
| Apps WITH onboarding campaigns Day 2 activation | 20% | Airship Q2 2024 |
| Apps WITHOUT onboarding campaigns Day 2 activation | 16% | Airship Q2 2024 |
| Effective onboarding retention uplift | up to 50% | Localytics |
| Users who abandon after single session | ~25% | Multiple sources |
| Critical retention window | First 3-7 days | Industry consensus |

### Community/Social App Examples

#### OLIO (Community Sharing / Anti-Waste)
- **Onboarding**: Previously had a problematic onboarding with too many food images and unclear brand mission. Redesigned to focus on landing screen and item listing flow instead.
- **Key insight from OLIO's own research**: The real issue was not onboarding friction but post-onboarding confusion -- users didn't understand how to list items or what the landing screen showed.
- **Current approach**: Minimal onboarding, quick value demonstration via the feed of available items, focus on making the "add item" flow (Snap. Message. Share.) dead simple.
- **Lesson for Offrii**: Post-onboarding experience matters more than onboarding itself. Make the core loop (browse wishes / offer gifts) immediately understandable.

#### Nextdoor (Neighborhood Community)
- **Onboarding**: Auth-first by necessity (address verification required for community trust)
- **Flow**: Sign up -> Enter address -> Verify address (postcard/phone/email) -> Neighborhood assigned -> Feed
- **Key insight**: For location-based community apps, address/location verification IS the onboarding -- it is what creates the value (connecting to your neighbors)
- **Lesson for Offrii**: Location is critical for a local gifting app. Make location setup feel like "unlocking your community" not "giving away your data."

#### Vinted (Second-Hand Fashion Marketplace)
- **Onboarding**: 9-screen flow including value proposition, account creation, and preference selection
- **Approach**: Category preference selection during onboarding to personalize the feed immediately
- **Lesson for Offrii**: Asking users what categories of items they are interested in (baby gear, books, furniture, clothing) during onboarding personalizes the experience from screen one.

#### BuyNothing (Community Gifting)
- **Onboarding**: Originally Facebook Groups (no app onboarding). The dedicated app uses location-based group matching.
- **Flow**: Sign up -> Location -> Join local group -> See local posts
- **Groups limited to 300-400 members** for community intimacy
- **Key insight**: The onboarding IS community discovery. Finding your local group is the "aha moment."
- **Lesson for Offrii**: Surface the local community immediately. Show users that their neighbors are already active.

#### GoFundMe (Community Fundraising)
- **Onboarding**: Minimal barrier. Users can browse campaigns without auth. Auth only required to donate or create.
- **"Try before you sign up" pattern** in action
- **Lesson for Offrii**: Let users browse wishes and available gifts before requiring sign-up.

### Apple's Recommendations on Onboarding

From HIG and WWDC sessions, Apple's guidance is consistent:
1. **Make it fast, fun, and educational**
2. **Provide an onboarding experience only when necessary** -- if the app is intuitive, skip it
3. **Give people the chance to skip** onboarding at any point
4. **Defer asking for data** until the app needs it
5. **Avoid asking for setup information that can be gathered from the system** (like location -- use Core Location)
6. **Show, don't tell** -- let users learn by doing

---

## 4. Auth Flow Placement

### Before vs After Onboarding

| Pattern | When to Use | Examples |
|---------|-------------|---------|
| **Auth BEFORE onboarding** | When the app requires a community identity (social, messaging) or address verification | Nextdoor, WhatsApp, Peach |
| **Auth AFTER onboarding** | When you want to demonstrate value before asking for commitment | GoFundMe, DoorDash (guest mode) |
| **Auth DURING onboarding** | When onboarding collects data that feeds into account creation | MyFitnessPal (goals -> account), Vinted (preferences -> account) |
| **Deferred auth (try-before-signup)** | When core value can be experienced without an account | Shopping apps, content apps |

### "Try Before You Sign Up" Pattern

This is a growing best practice, especially for marketplace and community apps:

**How DoorDash does it:**
- Users can browse restaurants, add items to cart without any account
- Auth is only triggered when they try to check out
- By this point, the user is already invested (items in cart)
- Result: Higher conversion because the user has already experienced value

**Applicability to Offrii:**
- **Browsable without auth**: Let users see available gifts/offers in their area and browse community wishes
- **Auth trigger points**: When user tries to (a) post a wish, (b) offer a gift, (c) message someone, (d) save/favorite an item
- **Risk**: Community trust may require knowing who is in the community. Mitigation: show content but blur or limit details until auth.

### Sign in with Apple -- Requirements & Best Practices

**Apple's App Store Rule (Mandatory):**
> If your app offers third-party sign-in (Google, Facebook, etc.), you MUST also offer Sign in with Apple.

**Implementation details:**
- Sign in with Apple provides: user ID (always), email (first sign-in only, may be relay address), full name (first sign-in only)
- Email and name are ONLY provided on first sign-in -- store them immediately
- Apple provides a stable `sub` (subject) identifier that persists across sessions
- Offer both "Sign in with Apple" and "Sign up with Apple" button variants as appropriate
- Place the Sign in with Apple button **first** or at the **top** of auth options (Apple's HIG requirement)
- Use the system-provided button styles (black, white, white outline) -- do not customize

**Recommended auth option hierarchy for Offrii:**
1. Sign in with Apple (required if offering other social login)
2. Continue with Google
3. Continue with Email (magic link preferred over password)
4. Phone number (optional, useful for community trust)

### Guest Mode Patterns

For Offrii, a limited guest mode is recommended:
- **Can do as guest**: Browse nearby offers, view community wishes, see community activity
- **Cannot do as guest**: Post wishes, offer gifts, message users, save favorites
- **Gentle nudge**: Show what they are missing with contextual prompts ("Sign up to message this neighbor about the bookshelf!")
- **Apple guideline 5.1.1**: Apple requires that if account features provide meaningful value beyond basic functionality, registration can be required -- but you must justify it. Community trust/safety is a valid justification.

---

## 5. Permission Requests

### Apple's Guidelines on Timing

Apple's HIG is clear on permission timing:

> "Only request a permission at app launch if it's necessary for the core functioning of your app."

**The framework:**

| Permission Criticality | Benefit Clarity | Strategy |
|-----------------------|-----------------|----------|
| Critical + Clear | Ask up-front | Location in a navigation app |
| Critical + Unclear | Educate up-front, then ask | Tracking permission (ATT) |
| Secondary + Clear | Ask in context | Camera when user taps "take photo" |
| Secondary + Unclear | Educate in context, then ask | Contacts for "find friends" |

### Permission Priming (Pre-Permission Screens)

**What it is**: Show a custom screen explaining WHY you need a permission BEFORE triggering the native iOS dialog.

**Why it matters**: The native iOS permission dialog can only be shown ONCE. If denied, the user must go to Settings to re-enable. A pre-permission screen lets you:
- Explain the benefit clearly
- Let the user opt out gracefully without triggering the system dialog
- Achieve dramatically higher grant rates

**Real data:**
- Cluster app: pre-permission overlays led to "nearly every user" granting contact access (vs much lower without)
- ATT pre-prompt education: 40-60% higher acceptance rates vs immediate system prompts (Secure Privacy, 2025)

### Permission Timing for Offrii

| Permission | When to Ask | Pre-Permission Message |
|-----------|-------------|----------------------|
| **Location** | During onboarding (core to the app) | "Offrii connects you with your neighbors. Allow location to discover gifts and wishes near you." |
| **Notifications** | After first meaningful action (e.g., after posting a wish or offering a gift) | "Get notified when a neighbor has something you wished for, or when someone wants your offer." |
| **Camera** | When user taps "Add photo" to list an item | "Take a photo of the item you want to give away" (no pre-screen needed -- context is obvious) |
| **Contacts** | Only if "invite friends" feature exists, when user explicitly taps "Invite" | "Find friends already on Offrii in your neighborhood" |
| **ATT (Tracking)** | After user has experienced value, ideally day 2+ or after 3+ sessions | "Help us improve your experience by allowing personalized recommendations" |

### Key Rules
1. NEVER batch all permissions at launch
2. Location can be asked early IF the app is fundamentally location-based (Offrii qualifies)
3. Notifications should wait until the user understands WHAT they will be notified about
4. Always provide clear, specific purpose strings in Info.plist (required by Apple)
5. Write purpose strings at an 8th-grade reading level (Apple requirement for ATT)

---

## 6. Post-Auth First Screen

### The Empty State Problem

For a new user who just signed up, every screen is empty: no wishes, no offers, no messages, no activity. This is the most dangerous moment in the user journey.

**The research says:**
- "Your empty state should never feel empty" -- Every Interaction (UX research)
- Conversational copy outperforms system copy by 20-40% in conversion rates
- Relevant illustrations increase engagement; purely decorative ones hurt conversion by up to 30%
- A single strong CTA is essential

### Empty State Design Best Practices

**DO:**
- Show a friendly greeting with the user's name
- Explain what this screen WILL contain once active
- Provide ONE clear action the user can take right now
- Use illustrations that relate to the missing content
- Show social proof ("12 neighbors are sharing near you")
- Pre-populate with nearby community content if possible

**DON'T:**
- Show "No items found" or blank screens
- Present multiple competing CTAs
- Use generic placeholder text
- Leave the user wondering "now what?"

### Quick Wins -- Getting the User Engaged Immediately

For Offrii, the post-auth first screen strategy:

1. **Feed Pre-Population**: Show nearby community activity IMMEDIATELY. Even if the user hasn't posted anything, show what neighbors are offering and wishing for. This is the #1 way to demonstrate value.

2. **First Action Prompt**: A prominent, friendly CTA:
   - "What do you wish for?" (post first wish) -- OR
   - "Have something to give?" (list first offer)
   - Make this feel like a 30-second action, not a commitment

3. **Community Counter**: "23 neighbors are already sharing on Offrii in [Neighborhood Name]" -- social proof that reduces the "empty community" fear

4. **Suggested Wishes**: Show popular wish categories to browse, lowering the barrier to finding something interesting

5. **Checklist/Progress**: Optional lightweight checklist:
   - [x] Create account
   - [ ] Post your first wish
   - [ ] Browse nearby offers
   - [ ] Complete your profile
   This gives direction without being forceful.

### The "Aha Moment" for Offrii

The activation moment -- the point where a user "gets it" and is likely to return:
- **Seeing a real nearby offer** that matches their interests
- **Receiving a response** to their posted wish
- **Successfully giving away an item** and feeling the community gratitude

Target: Get users to their aha moment within 60 seconds of entering the app.

---

## 7. Returning User Experience

### What Should Happen When a Returning Authenticated User Opens the App

**Ideal flow (< 2 seconds perceived):**
1. Static launch screen appears instantly (0ms)
2. App checks auth token in Keychain (background, non-blocking)
3. Show last-viewed screen with **cached data** immediately (skeleton or real cached content)
4. Silently refresh token if expired (background)
5. Fetch fresh data and update UI seamlessly (no loading spinners on main content)
6. Show badge/indicator for new activity (new wishes matching theirs, new messages, new offers nearby)

### Token Refresh Strategy

```
App Launch
    |
    v
Load cached data + show UI immediately
    |
    v (parallel)
Check access token expiration
    |
    +-- Not expired --> Use for API calls
    +-- Expired --> Use refresh token to get new access token
                      |
                      +-- Success --> Continue seamlessly
                      +-- Failure --> Show non-intrusive "Session expired" -> Login
```

**Key principles:**
- NEVER show a blank screen while checking auth
- NEVER block UI on token refresh
- Cache the last-known good state of all main screens
- Use refresh tokens stored in Keychain
- Handle refresh failure gracefully (don't crash, don't lose cached content)

### Skeleton Screens vs Splash Screens for Returning Users

**Research findings (LogRocket 2025, UX Collective):**
- Users perceive skeleton screens as **20-30% faster** than spinners for identical wait times
- Replacing spinners with skeletons: bounce rate during loading decreased 18%, user complaints about "slow site" down 60%
- Skeleton screens performed best on **emotional level** -- users were happiest with skeleton loading, least happy with blank screens
- Facebook is the gold standard: their launch screen matches the app structure, then populates with skeleton placeholders before real content fills in

**When to use skeletons:**
- Content feeds (wish lists, offer lists, activity feed) -- YES
- Profile screens with known structure -- YES
- Simple screens with <500ms load time -- NO (just show content)
- Screens where content structure is unpredictable -- NO (use spinner)

### Data Prefetch Strategy

For Offrii returning users:
1. **On launch**: Load cached feed data immediately from local storage (Core Data/SwiftData or JSON cache)
2. **Background fetch**: Prefetch updated feed, new messages count, notification badges
3. **Smart caching**: Cache the first 20 items of the main feed, user profile, and neighborhood data
4. **Incremental updates**: Use `If-Modified-Since` or pagination cursors to fetch only new content
5. **Image caching**: Use URLCache or SDWebImage/Kingfisher for aggressive image caching
6. **Background App Refresh**: Register for background fetch to update content even when app is not active

---

## 8. App Startup Performance

### Perceived vs Actual Performance

The most important insight from all the research: **perceived performance matters more than actual performance.**

Users who see a skeleton screen perceive the same 2.5-second load as significantly faster than users who see a spinner. The perceived speed difference is 20-30%.

### Best Practices for Fast Perceived Loading

1. **Instant launch screen** (iOS handles this with LaunchScreen.storyboard)
2. **Pre-warm critical data paths**: Start network requests in `application(_:didFinishLaunchingWithOptions:)` or app init, not in the first view's `onAppear`
3. **Show cached content first**: Display the last-known state while fresh data loads
4. **Skeleton screens for structured content**: Use placeholders that match the final layout
5. **Lazy loading for non-critical content**: Profile images, secondary sections, and settings can load after the main feed
6. **Avoid synchronous work on main thread**: Move all Keychain reads, UserDefaults checks, and network calls off the main thread
7. **App load time target**: Under 2 seconds to interactive state (Google/Apple recommendation)
8. **Time to first meaningful content**: Under 3 seconds

### Technical Optimizations

- **Reduce launch-time work**: Defer non-essential SDK initialization (analytics, crash reporting can init asynchronously)
- **Use `@MainActor` wisely**: Only UI updates on main actor; data loading on background tasks
- **Precompile views**: SwiftUI view body computation should be minimal; heavy work in view models
- **Network connection pre-warming**: Start a URLSession data task early to warm up TCP/TLS connections
- **Minimize dynamic linking**: Reduce framework count; merge smaller frameworks
- **Profile with Instruments**: Use Time Profiler and App Launch template to identify actual bottlenecks

---

## 9. Recommended Flow for Offrii

### Complete Launch-to-Engagement Flow

```
FIRST-TIME USER:
===============

1. [0ms]     Static Launch Screen (Offrii logo on brand color, tab bar outline)
2. [~500ms]  Brief brand animation / transition
3. [~1s]     Value Proposition Screens (2-3 screens max):
               - Screen 1: "Discover what your neighbors are sharing" (with illustration)
               - Screen 2: "Post a wish, find it locally" (with illustration)
               - Screen 3: "Give things a second life" (with illustration)
             [Skip button always visible] [Dots indicator for progress]
4. [User]    Location Permission (pre-primed):
               "Offrii connects you with your local community.
                Allow location to see gifts and wishes near you."
               [Continue] [Not Now]
5. [User]    Auth Screen:
               - "Join your neighborhood" (headline)
               - [Sign in with Apple] (first, prominent)
               - [Continue with Google]
               - [Continue with Email]
               - [Browse as Guest] (smaller, bottom)
               - "Already have an account? Sign in" (link)
6. [User]    Quick Personalization (1 screen, optional):
               "What are you interested in?"
               [Baby & Kids] [Books] [Furniture] [Clothing] [Electronics] [Kitchen] [Garden] [Other]
               (Multi-select chips, skip-able)
7. [Instant] Main Feed -- Pre-populated with nearby community activity:
               - Header: "Welcome to [Neighborhood]! 23 neighbors are sharing."
               - Feed of nearby offers and wishes
               - Floating action button: "+" to post wish or offer
               - Bottom sheet (first time only): "Post your first wish in 30 seconds"
8. [After    Notification Permission (after first meaningful action):
    first     "Get notified when a neighbor has what you wished for"
    action]   [Enable Notifications] [Maybe Later]


RETURNING AUTHENTICATED USER:
==============================

1. [0ms]     Static Launch Screen (same as first time)
2. [~200ms]  Main Feed with cached data (or skeleton if no cache)
3. [~500ms]  Fresh data loads in, UI updates seamlessly
4. [~1s]     Badges update (new messages, wish matches, nearby offers)
             No interruptions. No splash. No delays.


RETURNING UNAUTHENTICATED USER (logged out or token failure):
=============================================================

1. [0ms]     Static Launch Screen
2. [~500ms]  "Welcome back!" Login screen
               - [Sign in with Apple]
               - [Continue with Google]
               - [Continue with Email]
               - No onboarding screens (they have seen them)
3. [Auth]    Main Feed (with fresh data fetch)
```

### Key Design Principles for Offrii

1. **Value before commitment**: Show the community feed (even partially) before requiring auth when feasible
2. **Location is the value unlock**: Frame location permission as "discovering your community" not "tracking you"
3. **Minimize onboarding screens**: 2-3 value proposition screens maximum; never feature tutorials upfront
4. **Progressive disclosure for features**: Explain "how to post a wish" the first time the user taps "+", not during onboarding
5. **Social proof early**: Show neighbor count, recent activity, and community warmth from the very first screen
6. **Empty states are opportunities**: Every empty screen should guide toward a productive action
7. **Cache aggressively for returning users**: The app should feel instant on subsequent launches
8. **Skeleton screens for feeds**: Use shimmer placeholders that match the wish/offer card layout
9. **Permission timing matters**: Location early (core value), notifications after first action, camera on demand
10. **Guest browsing builds desire**: Letting guests see what is available makes them want to join

---

## Sources

### Apple Documentation
- Apple HIG: Launching - developer.apple.com/design/human-interface-guidelines/launching
- Apple HIG: Loading - developer.apple.com/design/human-interface-guidelines/loading
- Apple HIG: Sign in with Apple - developer.apple.com/design/human-interface-guidelines/sign-in-with-apple
- Apple App Store Review Guidelines - developer.apple.com/app-store/review/guidelines

### Retention & Benchmark Data
- Adjust: "What Makes a Good Retention Rate" (2024) - adjust.com/blog/what-makes-a-good-retention-rate
- Airship: Q2 2024 Onboarding Campaign Benchmarks (via eMarketer, Jun 2025)
- AppSamurai: Day 1 Retention Benchmarks by Vertical (2025) - appsamurai.com
- Mobile App Trends 2025: Marketplace retention data
- Growth-onomics: Mobile App Retention Benchmarks 2026 - growth-onomics.com

### Onboarding Best Practices
- VWO: Ultimate Mobile App Onboarding Guide (2026) - vwo.com/blog/mobile-app-onboarding-guide
- Adapty: Mobile App Onboarding Best Practices - adapty.io/blog/mobile-app-onboarding
- NextNative: 7 Mobile Onboarding Best Practices (2025) - nextnative.dev
- GuideJar: User Onboarding Best Practices (2025) - guidejar.com
- Eleken: User Onboarding Best Practices - eleken.co/blog-posts/user-onboarding-best-practices
- UXCam: App Onboarding Flow Examples (2026) - uxcam.com/blog/10-apps-with-great-user-onboarding
- DesignerUp: "I studied the UX/UI of over 200 onboarding flows" - designerup.co

### Launch Screens & Performance
- Mobbin: Launch Screen UI Pattern - mobbin.com/glossary/launch-screen
- SitePoint: How to Speed Up UX with Skeleton Screens - sitepoint.com
- LogRocket: Skeleton Loading Screen Design (Apr 2025) - blog.logrocket.com
- Medium/Mohit Phogat: Skeleton Screens perceived performance (Dec 2025)
- UX Collective: Skeleton Screens research study - uxdesign.cc

### Permissions & Privacy
- DogTown Media: Mobile Permission Requests Timing Guide - dogtownmedia.com
- Appcues: Permission Priming Strategies - appcues.com/blog/mobile-permission-priming
- Secure Privacy: Mobile App Consent iOS 2025 - secureprivacy.ai
- Apple Developer Blog: Best Practices for Permissions

### App Examples
- OLIO case study (Estelle Jin, GA redesign) - estellejin.com/olio-case-study
- Vinted onboarding flow - theappfuel.com
- Nextdoor onboarding - help.nextdoor.com
- DoorDash guest mode - referenced in Eleken blog

### Auth Implementation
- WorkOS: Sign in with Apple 2025 Requirements - workos.com/blog
- Auth0: Using Refresh Tokens in iOS Swift - auth0.com/blog
- Hacking with Swift: First App Launch Detection - hackingwithswift.com
- Stack Overflow: Detecting First Launch in iOS - stackoverflow.com
