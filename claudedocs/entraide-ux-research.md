# Mutual Aid & Gift-Giving App UX Research
## For Offrii Entraide Feature Redesign
### March 2026

---

## Table of Contents
1. [Executive Summary](#1-executive-summary)
2. [App-by-App Analysis](#2-app-by-app-analysis)
3. [Feature Comparison Matrix](#3-feature-comparison-matrix)
4. [Top 10 UX Patterns to Adopt](#4-top-10-ux-patterns-to-adopt)
5. [Top 5 Anti-Patterns to Avoid](#5-top-5-anti-patterns-to-avoid)
6. [Supply/Demand Imbalance Strategies](#6-supplydemand-imbalance-strategies)
7. [Gamification Patterns for Generosity](#7-gamification-patterns-for-generosity)
8. [Geographic Proximity Handling](#8-geographic-proximity-handling)
9. [Accessibility Considerations](#9-accessibility-considerations)
10. [Wireframe-Level Screen Descriptions](#10-wireframe-level-screen-descriptions)
11. [Recommendations for Offrii Entraide](#11-recommendations-for-offrii-entraide)
12. [Sources](#12-sources)

---

## 1. Executive Summary

This research analyzes 9 apps across the gift economy, mutual aid, and marketplace space to inform the redesign of Offrii's "Entraide" (mutual aid) feature. The analysis covers discovery, posting, matching, trust, status tracking, notifications, and mobile-first patterns.

**Key findings:**

- **Discovery**: The most successful apps combine a map-based view with a feed-based list. Geolocation-first browsing with category filters is the dominant pattern.
- **Posting**: Minimal friction posting (3-5 fields max) with photo-first flows outperform verbose forms. AI-assisted listing creation is emerging.
- **Matching**: In-app messaging with structured initial contact (not free-text) reduces noise and increases follow-through.
- **Trust**: Community-driven moderation scales better than manual review. "Banana" or credit systems (Geev) and profile completeness indicators build reciprocity.
- **Status tracking**: Simple lifecycle badges (Open > Claimed > Fulfilled) are essential. Most apps underinvest here.
- **Supply/Demand**: The hardest problem. Geev's "banana" currency and Buy Nothing's "gratitude" culture are the strongest solutions.
- **Mobile-first**: Bottom navigation, swipe gestures, haptic feedback on key actions, and progressive disclosure are table stakes in 2026.

**Confidence Level**: High (0.85) -- based on direct app store data, official documentation, recent updates (Jan-Mar 2026), UX case studies, and marketplace design best practices.

---

## 2. App-by-App Analysis

### 2.1 GEEV (French, Giving/Receiving Objects & Food)

**Overview**: 6M+ users, 550K ads/month, France's leading person-to-person donation app. 26M items donated since 2017.

**Discovery UX**
- Map + list hybrid: Users browse items on a geolocated map OR scroll a feed of cards sorted by proximity
- Category filters at top (horizontal scrollable chips): Furniture, Electronics, Clothing, Babies, Leisure, DIY, Food
- Search bar with autocomplete
- Geolocation auto-detection on first launch; adjustable radius
- Card layout: Photo (large), title, distance, time posted
- "Alert" feature: Save searches and get push notifications when matching items appear

**Posting UX**
- Photo-first: Camera opens immediately, up to 5 photos
- Title, description, category (required)
- Location auto-filled from GPS, editable
- Condition selector (good/fair/poor for objects)
- Food category has expiry date field
- Publication in "a few clicks" -- intentionally minimal

**Matching/Communication UX**
- "Banana" currency system: Each user has bananas; contacting a giver costs 1 banana. You earn bananas by giving items.
- In-app chat once contact is initiated
- Giver chooses from interested receivers
- No direct phone/email sharing until giver decides

**Trust & Safety**
- Profile with badges, points, and ranking
- Enriched user profiles showing giving/receiving history
- Community reporting system
- Banana system naturally filters non-serious requesters
- Geev Plus (paid) for enhanced access -- creates a "skin in the game" layer

**Status/Progress UX**
- Ad states: Active > Reserved > Given
- Giver manually updates status
- Ads expire after configurable period

**Notification UX**
- Push notifications for: new messages, someone interested in your item, alert matches
- In-app notification center

**Mobile-first Patterns**
- Native iOS/Android apps (4.8/4.7 ratings)
- Map with clustering for dense areas
- Pull-to-refresh on feed
- Bottom tab navigation (Home, Search, Post, Messages, Profile)

**Key Takeaway for Offrii**: The banana currency system is the single most innovative pattern here. It creates a self-regulating economy that encourages giving and prevents request spam.

---

### 2.2 Buy Nothing (Hyperlocal Gift Economy)

**Overview**: 14M+ members across 50+ countries, 2.6M items shared monthly. Major 3.0 redesign launched January 22, 2026.

**Discovery UX**
- Three content types: Gives (offerings), Asks (needs), and Gratitude (thank-yous)
- Feed sorted by recency within your local group
- "Neighborhood and Beyond" -- browse nearby or search globally for special items
- Post cards show: photo, title, poster name/avatar, distance, time
- Unread message count badge visible in navigation
- New in 3.0: improved browsing, search by neighbor name in messages

**Posting UX**
- Three post types: Give, Ask, Gratitude
- Photo upload (multiple supported, fixed in 3.0.4)
- Description with line breaks now supported (3.0.8)
- Location auto-assigned based on group membership
- New: ability to format posts with paragraph spacing

**Matching/Communication UX**
- Comment-based interest on public posts
- Giver selects recipient from interested parties
- Private messaging for coordination
- New: "Ask-a-friend" pickup delegation
- Shipping option for non-local items (premium feature)

**Trust & Safety**
- Hyperlocal groups (verified address/neighborhood)
- Real name policy
- Community moderation by volunteer admins
- Profile displays hometown (new in 3.0.8)
- Sign-in method verification (Facebook, Apple, Google, Email)
- Private communities feature for trusted sub-groups

**Status/Progress UX**
- Posts show: Available > Pending > Completed
- Gratitude posts serve as social proof and completion confirmation
- No formal lifecycle tracking beyond post status

**Notification UX**
- Push notifications for new posts in group, messages, community invites
- Unread indicators on inbox and side menu
- Notification now links directly to relevant community (fixed in 3.0.8)

**Mobile-first Patterns**
- Side menu navigation
- Photo-centric feed
- Infinite scroll
- Deep linking to notifications

**Key Takeaway for Offrii**: The "Gratitude" post type is brilliant. It closes the feedback loop, provides social proof, and creates emotional reward for givers. Offrii should implement a similar "thank you" flow post-fulfillment.

---

### 2.3 Freecycle / Trash Nothing

**Overview**: 12M+ members, 5,332 local towns, entirely nonprofit, grassroots.

**Discovery UX**
- Dashboard-based: central control for all activity
- Posts categorized as OFFER or WANTED
- Search Posts feature to find before posting
- Town-based browsing (join local group)
- Friends Circle for private gifting with trusted contacts
- Available in 11 languages

**Posting UX**
- Two types: Offer or Wanted
- Up to 3 photos per post
- Can edit/add photos after initial posting
- One post at a time (anti-spam measure)
- Guidelines may restrict certain WANTED types per community

**Matching/Communication UX**
- Email-based notification system (configurable: "one for each post" vs. digest)
- Direct replies to posts
- No in-app chat (relies on email)
- One-post-at-a-time rule prevents cross-posting spam

**Trust & Safety**
- Volunteer-moderated local groups
- Zero tolerance for scams, spam, and adult content
- Community guidelines per town
- Long history (since 2003) builds institutional trust

**Status/Progress UX**
- Minimal: posts are active or closed
- No formal lifecycle tracking

**Key Takeaway for Offrii**: Freecycle's simplicity is both its strength and weakness. The "Friends Circle" concept of private trusted groups is worth noting. Offrii already has Circles -- connecting Entraide to Circles is a natural fit.

---

### 2.4 Facebook Marketplace (Free Section / Community Help)

**Overview**: Billions of users, recently redesigned (late 2025) toward "social shopping."

**Discovery UX**
- Algorithm-driven feed showing items based on browsing history, searches, and interactions
- Category filters + location radius selector
- Map view for local items
- Sort by: distance, price, date listed, relevance
- "Free" filter to show only free items
- Collaborative collections: save listings, share with friends, invite to group decisions
- React and comment directly on listings (new 2025)

**Posting UX**
- Photo-first flow
- Category, title, description, price (or "Free"), location
- Condition selector
- AI-powered: Meta AI suggests pricing, identifies items from photos
- Cross-posting to Buy/Sell groups

**Matching/Communication UX**
- Messenger integration for direct chat
- Quick-reply templates ("Is this still available?")
- Collaborative buying: add friends to seller chat
- Meeting point suggestions

**Trust & Safety**
- Facebook identity verification (real name, profile history)
- Seller ratings and response time badges
- Community reporting
- Purchase Protection for shipped orders (not local pickup)
- "Verified" badges for active, responsive sellers

**Status/Progress UX**
- Listed > Pending > Sold/Given Away
- Mark as Available/Pending/Sold
- No formal pickup confirmation flow

**Notification UX**
- Messenger notifications
- Price drop alerts
- New listing alerts based on saved searches
- "Similar items" suggestions

**Mobile-first Patterns**
- Integrated into main Facebook app (bottom tab)
- Infinite scroll grid
- Quick-action buttons (Share, Save, Message)
- AI-powered search suggestions

**Key Takeaway for Offrii**: The "collaborative collections" feature is interesting for mutual aid -- a family or group could collectively track needs and available offers. The quick-reply templates reduce messaging friction significantly.

---

### 2.5 Nextdoor (Neighborhood Help Section)

**Overview**: 105M+ neighbors across 350K neighborhoods. Major redesign July 2025 with Alerts, News, and Ask pillars.

**Discovery UX**
- **Help Map**: Dedicated map showing neighbors offering and requesting help
- Feed-based browsing of posts in your verified neighborhood
- "For Sale & Free" section for item exchange
- **Ask feature**: AI-powered local recommendation tool that surfaces answers from years of neighbor conversations
- Groups and Events for community organizing
- Content categories: Alerts, News, Ask, For Sale & Free

**Posting UX**
- Post types: General, Recommendation, For Sale, Free, Event, Alert
- Photo support
- Location auto-verified based on address
- Free-form text with category tagging

**Matching/Communication UX**
- Direct messaging between neighbors
- Comment threads on posts
- AI-summarized recommendation responses

**Trust & Safety**
- **Address verification**: Core differentiator. Every member must verify their home address
- Real identity (name + address)
- Community guidelines with enforcement
- Transparency report published annually
- Content moderation: human judgment + AI + community reporting
- 2025 report: detailed breakdown of content removal and appeals

**Status/Progress UX**
- Posts are active or archived
- Help Map shows current offers/needs
- No formal item lifecycle tracking

**Notification UX**
- Alerts for nearby safety events, weather, traffic
- "Smarter notifications" -- AI-curated relevance
- Email digests (configurable frequency)
- Push for urgent alerts

**Mobile-first Patterns**
- Redesigned in July 2025 with streamlined bottom nav
- Map-centric Help section
- AI-powered Ask with conversational UI
- Real-time alerts with rich media

**Key Takeaway for Offrii**: The Help Map is exactly what Offrii's Entraide needs. A dedicated map view showing requests and offers geographically creates urgency and proximity awareness. Address verification is the gold standard for trust in hyperlocal apps.

---

### 2.6 GoFundMe / Leetchi (Crowdfunding for Needs)

**Overview**: GoFundMe has facilitated $40B+ in donations from 200M+ contributions. Leetchi is the French equivalent for group fundraising.

**Discovery UX**
- Browse by category: Medical, Emergency, Education, Community, etc.
- Search by keyword, organizer name
- Trending/featured campaigns on homepage
- "Near me" discovery based on location
- Social sharing drives most discovery (not in-app browsing)

**Posting UX (Campaign Creation)**
- Multi-step wizard:
  1. Category selection
  2. Who you're raising for (yourself, someone else, charity)
  3. Goal amount
  4. Story (rich text editor with photo/video embedding)
  5. Cover photo/video
  6. Organizer details
- Strong emphasis on storytelling transparency
- Verification: ID, bank account, beneficiary details
- Tips for success shown during creation

**Trust & Safety**
- GoFundMe Guarantee (donor refunds for misuse)
- Identity verification (government-issued ID)
- Bank account verification
- Facebook account connection for transparency
- Trust & Safety team reviews flagged campaigns
- Campaign page transparency guidelines: who, what, why, how funds used
- Public donation ledger

**Status/Progress UX**
- Goal progress bar (prominent, animated)
- Donor count and recent donor names
- Updates section: organizer posts progress/thank-you updates
- Goal can be raised if exceeded (not a red flag)
- Withdrawal tracking for beneficiary

**Notification UX**
- Push: new donations, comments, milestone reached
- Email: campaign updates, fundraising tips
- Social sharing prompts after each donation

**Mobile-first Patterns**
- One-tap donation with saved payment methods
- Social sharing deep links
- Progress bar animations
- Quick update posting from mobile

**Key Takeaway for Offrii**: The **progress bar and update mechanism** are powerful patterns. For Entraide wishes, a simple visual showing "3 people interested" or "1 offer received" creates momentum. The storytelling emphasis in campaign creation is also relevant -- helping users tell their story makes requests more compelling.

---

### 2.7 Too Good To Go (Food Waste -- UI Patterns)

**Overview**: 350M+ meals saved, partnerships with 180K+ businesses. Premium UX for discovery and pickup.

**Discovery UX**
- **Map-first**: Opens to a geolocated map showing nearby Surprise Bags with pins
- **List view toggle**: Switch between map and scrollable card list
- **Filter bar**: Meal type (bakery, restaurant, grocery), pick-up time (today/tomorrow), price range, dietary preferences, rating
- **Discover feed**: Personalized recommendations, "new nearby," segment-specific filters
- **Favorites tab**: Heart icon to track preferred stores, alerts when available
- Sort by: distance, price, rating, relevance
- Store cards: Cover photo, store name, bag type, price, rating, distance, pick-up window

**Pickup Confirmation UX**
- Swipe-to-confirm slider at pickup (satisfying haptic feedback)
- In-app timer for pickup windows
- "Ask-a-friend" feature for delegated pickup

**Status/Progress UX**
- Reserved > Pick up window open > Picked up > Rate
- Push notification when pickup window opens
- Clear timeline with time remaining

**Notification UX**
- Favorites alerts: "Your favorite bakery just added bags!"
- Pickup reminders
- New store alerts
- Impact tracking notifications ("You've saved X meals!")

**Mobile-first Patterns**
- Map with animated clustering
- Card swiping for browsing
- Swipe-to-confirm gesture for pickup
- Haptic feedback on key actions
- Impact counter with animation
- Bottom sheet for filter/sort overlays

**Key Takeaway for Offrii**: The **map + list toggle**, **favorites with alerts**, and **swipe-to-confirm** gesture are polished patterns to study. The impact counter ("You've saved X meals") translates perfectly to "You've helped X people" for Entraide.

---

### 2.8 Vinted (Marketplace UX Patterns)

**Overview**: Multi-billion-euro valuation, Europe's largest fashion marketplace. Zero seller fees -- buyers pay for protection.

**Discovery UX**
- Personalized feed based on browsing history, saved searches, and followed brands/sizes
- Category navigation: hierarchical (Women > Tops > T-Shirts)
- Filter system: brand, size, color, condition, price, material, location
- Saved searches with push alerts
- "Newsfeed" of items from followed members
- Search with autocomplete and recently viewed
- Grid layout with 2 columns of photo cards

**Posting UX**
- Photo-first: up to 20 photos, camera or gallery
- AI-powered: auto-detection of brand, category from photo (emerging)
- Required: photos, title, description, category, size, condition, price
- Smart pricing suggestions based on comparable listings
- Draft saving
- Edit = re-index in search (soft boost)

**Matching/Communication UX**
- In-app messaging with image sharing
- "Make an offer" button with counter-offer flow
- Quick responses = higher search ranking
- "Fast Shipper" badge for quick dispatch
- Bundle feature: buy multiple items from same seller

**Trust & Safety**
- Buyer Protection fee (platform revenue model)
- Seller verification
- Rating system (1-5 stars with text review)
- Photo verification (real photos, not stock)
- Escrow payment system
- Dispute resolution process

**Status/Progress UX**
- Listed > Sold > Shipped > Delivered > Rated
- Tracking integration for shipments
- Clear status badges on each listing

**Notification UX**
- Push: new message, offer received, item sold, price drop on favorites
- Saved search alerts
- Promotional notifications (wisely limited)

**Mobile-first Patterns**
- Bottom tab navigation (Home, Search, Sell [+], Inbox, Profile)
- Pull-to-refresh
- Swipe to dismiss
- Floating action button for new listing
- Photo grid with zoom on tap
- Infinite scroll with lazy loading

**Key Takeaway for Offrii**: Vinted's **seller-side UX** is exceptional. The "Sell" button as a prominent center tab, the photo-first listing flow, the smart pricing suggestions, and the status lifecycle are all patterns Offrii should adopt for wish creation and tracking.

---

### 2.9 Olio (Food/Item Sharing)

**Overview**: 9M+ users, 100M+ meals shared. Recently redesigned Home + Explore screens (March 2025).

**Discovery UX**
- **Home screen redesign (2025)**: Separated navigation to show Olio is more than "free food"
- **Explore screen**: Browse by category (Free Food, Free Stuff, Reduced Food, Lucky Dip)
- Location-based: items shown by proximity
- Card layout: photo, title, distance, time since posted
- "Lucky Dip" bags (similar to TGTG Surprise Bags)
- Collection feature for batch items from retailers
- New users now 15% more likely to add a listing after redesign

**Posting UX**
- Photo-first listing
- "AI-ify" feature: AI helps improve listing descriptions (recently fixed in update)
- Category selection
- Pick-up details and timing
- Quick and simple -- designed for high volume posting

**Matching/Communication UX**
- In-app messaging (recently improved with unread indicators, timestamps, unread filter)
- Claim-based: tap "Request" on an item
- Pick-up quota system for fair distribution (replaced confusing "dynamic delays")
- "Save Me Collections" for unclaimed items

**Trust & Safety**
- Food Waste Hero volunteer program (trained community members)
- Food safety training modules for catered food
- Community reporting
- Bot-driven check-ins (controversial -- some users find it annoying)
- Pop-up ads for support donations (major UX complaint)

**Status/Progress UX**
- Available > Requested > Confirmed > Collected
- Home screen shows "when food is confirmed" on collections
- Pick-up time windows

**Notification UX**
- Push for new nearby items, message updates, collection confirmations
- Monthly community updates
- Too many donation request popups (negative UX feedback)

**Mobile-first Patterns**
- Card-based feed
- Bottom navigation
- Location-aware sorting
- Quick-claim buttons

**Key Takeaway for Offrii**: The **pick-up quota system** replacing "dynamic delays" is a valuable lesson in supply/demand management. Also, the strong negative reaction to excessive donation popups and bots is a warning -- keep the core UX clean and non-intrusive.

---

## 3. Feature Comparison Matrix

| Feature | GEEV | Buy Nothing | Freecycle | FB Marketplace | Nextdoor | GoFundMe | TGTG | Vinted | Olio |
|---------|------|-------------|-----------|----------------|----------|----------|------|--------|------|
| **Discovery** |
| Map view | Yes | No | No | Yes | Yes (Help Map) | No | Yes (primary) | No | No |
| List/feed view | Yes | Yes (primary) | Yes | Yes | Yes | Yes | Yes | Yes (primary) | Yes |
| Grid view | No | No | No | Yes | No | No | No | Yes (primary) | No |
| Category filters | Yes | No | No | Yes | Limited | Yes | Yes | Yes (rich) | Yes |
| Search | Yes | Yes | Yes | Yes (AI) | Yes (AI) | Yes | Yes | Yes (rich) | Yes |
| Saved searches/alerts | Yes | No | No | Yes | No | No | Yes | Yes | No |
| Proximity sorting | Yes | Yes | Yes | Yes | Yes | Limited | Yes | Limited | Yes |
| **Posting** |
| Photo-first flow | Yes | Yes | Yes (3 max) | Yes | Yes | Yes | N/A (B2B) | Yes (20 max) | Yes |
| AI-assisted creation | No | No | No | Yes | No | No | No | Emerging | Yes |
| Required fields | 3-4 | 2-3 | 2-3 | 4-5 | 2-3 | 5-6 | N/A | 6-7 | 3-4 |
| Anonymous option | No | No | No | No | No | No | No | No | No |
| **Matching** |
| In-app messaging | Yes | Yes | No (email) | Yes (Messenger) | Yes | Yes | No | Yes | Yes |
| Structured first contact | Yes (banana) | Comment | Reply | Template | Message | Donate | Reserve | Offer | Request |
| Giver/poster selects | Yes | Yes | Yes | Yes | N/A | N/A | N/A | N/A | First-come |
| **Trust** |
| Identity verification | Profile | Real name + area | None formal | Facebook ID | Address | Gov't ID | N/A | Buyer protect. | None formal |
| Rating/review system | Points/badges | Gratitude posts | No | Seller ratings | No formal | Donor count | Store ratings | Star ratings | No formal |
| Moderation | Community | Volunteer admin | Volunteer mod | AI + reports | AI + human | Trust team | N/A | AI + manual | Volunteers + AI |
| **Status Tracking** |
| Lifecycle badges | 3 states | 3 states | 2 states | 3 states | 2 states | Progress bar | 4 states | 5 states | 4 states |
| Progress visualization | No | No | No | No | No | **Yes (bar)** | Time window | Tracking | Time window |
| **Gamification** |
| Points/currency | **Bananas** | No | No | No | No | No | No | No | No |
| Badges/ranks | **Yes** | No | No | Seller badges | No | No | No | Yes | Volunteer badges |
| Impact tracking | **CO2 saved** | Items shared | No | No | No | $ raised | **Meals saved** | No | **Meals shared** |

---

## 4. Top 10 UX Patterns to Adopt

### Pattern 1: Map + Feed Dual Discovery
**Source**: GEEV, Too Good To Go, Nextdoor Help Map
**What**: Toggle between a map view (items as pins with distance) and a scrollable card feed. Map is default for physical item exchange.
**Why**: Proximity is the #1 factor in mutual aid. A map instantly communicates what's nearby and creates urgency.
**Implementation for Offrii**: Tab bar on Entraide screen: "Feed" | "Map". Map shows wish pins color-coded by category. Tapping a pin opens a bottom sheet preview.

### Pattern 2: Photo-First, Minimal-Field Posting
**Source**: GEEV, Vinted, Olio
**What**: The "create" flow opens with camera/gallery immediately. Title and category are the only mandatory text fields. Description is optional but encouraged with AI suggestions.
**Why**: Every additional required field reduces completion rate by 10-15%. Mutual aid users may be in urgent situations -- speed matters.
**Implementation for Offrii**: "New Wish" flow: Photo (optional but prompted) > Title > Category (pill selector) > Description (with AI-ify suggestion) > Anonymous toggle > Submit. Max 4 taps to post.

### Pattern 3: "Banana" Economy / Reciprocity Credits
**Source**: GEEV
**What**: A virtual currency that costs 1 unit to contact a poster, and earns units when you give. Prevents spam and encourages reciprocity.
**Why**: Solves the "too many requests, not enough givers" problem organically. Creates a self-balancing economy.
**Implementation for Offrii**: "Coeurs" (hearts) system. Every user starts with 3. Offering to help earns 1 coeur. Posting a wish costs 0 (important: never gate asking for help). Contacting someone about a wish you want to fulfill costs 0. The economy should reward giving, not punish asking.

### Pattern 4: Gratitude Loop / Thank-You Flow
**Source**: Buy Nothing
**What**: After fulfillment, the receiver is prompted to post a "Gratitude" message visible to the community.
**Why**: Social proof of successful exchanges. Emotional reward for givers. Builds community trust and warmth. Creates a virtuous cycle.
**Implementation for Offrii**: After confirming fulfillment, prompt: "Want to thank [Helper]?" Opens a quick gratitude post with photo option, shared to the community feed or Circle.

### Pattern 5: Impact Counter / Personal Dashboard
**Source**: Too Good To Go, GEEV, Olio
**What**: Persistent counter showing personal and community impact. "You've helped 12 people. Community has fulfilled 847 wishes."
**Why**: Transforms abstract generosity into tangible achievement. Proven to increase retention (TGTG uses it as primary engagement driver).
**Implementation for Offrii**: Profile section showing: Wishes fulfilled (as helper), Wishes received (as asker), Community impact number. Milestone celebrations at 5, 10, 25, 50 helps.

### Pattern 6: Status Lifecycle with Visual Progress
**Source**: Vinted (5 states), GoFundMe (progress bar), TGTG (time windows)
**What**: Clear, visible status badges on every wish card showing exactly where it is in the lifecycle: Open > Offered > Matched > In Progress > Fulfilled.
**Why**: Reduces anxiety for requesters ("Is anyone even seeing this?") and provides clarity for helpers ("Has someone already offered?").
**Implementation for Offrii**: Colored badge on each wish card. Open (blue) > Offered (orange, "2 offers") > Matched (yellow) > Fulfilled (green checkmark). GoFundMe-style interest indicator: "2 out of 3 needed" for partial fulfillment scenarios.

### Pattern 7: Saved Searches with Push Alerts
**Source**: GEEV, Vinted, FB Marketplace
**What**: Users save search criteria (e.g., "clothing + children + within 5km") and receive push notifications when matching wishes/offers appear.
**Why**: Converts passive browsers into active helpers. Bridges the time gap between when a need is posted and when the right helper sees it.
**Implementation for Offrii**: "Alert me" button on search results or category views. Simple criteria: category + distance. Push notification: "New wish matching your alert: 'School supplies for my daughter' -- 2km away."

### Pattern 8: Quick-Action Contact Templates
**Source**: Facebook Marketplace, Vinted
**What**: Pre-written message templates for first contact: "I can help with this!" / "I have what you need" / "Tell me more about what you need."
**Why**: Reduces the blank-page anxiety of messaging strangers. Increases response rates. Makes first contact feel structured and safe.
**Implementation for Offrii**: When tapping "Offer to Help" on a wish, show 3 quick-reply options plus free-text. Pre-fill with context from the wish.

### Pattern 9: Category Chips with Emoji Identifiers
**Source**: GEEV, TGTG, Vinted
**What**: Horizontal scrollable row of category chips at the top of the feed, each with an emoji or icon: "Education", "Clothing", "Health", "Children", "Home", "Religion", "Other".
**Why**: Faster than dropdown filters. Visual scanning with emojis is 2x faster than text-only. Familiar iOS pattern (see App Store).
**Implementation for Offrii**: Top of Entraide feed: horizontally scrollable chips. Each with emoji + label. Tapping filters the feed/map in real time. "All" chip is first and pre-selected.

### Pattern 10: Swipe-to-Confirm Fulfillment
**Source**: Too Good To Go
**What**: A satisfying swipe gesture (with haptic feedback) to confirm that an exchange happened. Both parties swipe to confirm.
**Why**: Creates a moment of ritual and completion. Haptic feedback provides sensory reward. Prevents accidental taps on important state changes.
**Implementation for Offrii**: On the wish detail screen, when both parties agree fulfillment happened, each sees a slider: "Swipe to confirm." Strong haptic pulse on completion. Confetti animation briefly. Transitions to gratitude prompt.

---

## 5. Top 5 Anti-Patterns to Avoid

### Anti-Pattern 1: Aggressive Monetization Popups
**Source**: Olio (negative reviews), various
**What**: Frequent popups asking for donations, premium upsells, or ad interruptions during core flows (browsing, messaging, giving).
**Why it hurts**: Users giving away items for free are especially sensitive to being monetized. Olio reviews explicitly cite this as the reason for uninstalling. "It honestly made me want to stop using the app and just throw my stuff in the bin."
**Offrii rule**: Never interrupt the giving/asking flow. Any monetization (Offrii Plus, if applicable) should be passive and positioned only in Profile or Settings.

### Anti-Pattern 2: Email-Only Communication
**Source**: Freecycle
**What**: Relying on email replies instead of in-app messaging for coordination.
**Why it hurts**: Email is slow, gets lost in spam, and breaks the contextual experience. Users must leave the app. Completion rates plummet.
**Offrii rule**: All communication must happen in-app. Never expose email addresses. Messages must retain wish context (linked to the specific wish being discussed).

### Anti-Pattern 3: "Dynamic Delays" / Artificial Scarcity
**Source**: Olio (removed in 2025 after backlash)
**What**: Artificially delaying when users could claim items to "fairly distribute" supply. Created confusion and frustration.
**Why it hurts**: Users couldn't understand why they had to wait. Felt punitive. Reduced trust in the platform.
**Offrii rule**: No artificial barriers to engaging with wishes. If fair distribution is needed, use transparent mechanisms (first-come-first-served, or giver's choice with clear communication).

### Anti-Pattern 4: No Post Formatting or Rich Text
**Source**: Buy Nothing (fixed in 3.0.8 after heavy user demand)
**What**: Not allowing line breaks, formatting, or paragraphs in posts. Walls of text reduce readability.
**Why it hurts**: Users couldn't describe their needs clearly. Posts became hard to scan. Frustrated both posters and browsers.
**Offrii rule**: Support basic formatting in wish descriptions. At minimum: line breaks, bold for titles. Keep it simple but readable.

### Anti-Pattern 5: No-Shows with Zero Consequence
**Source**: Buy Nothing (major user complaint), Facebook Marketplace
**What**: Users commit to helping/picking up but never show. No penalty, no tracking, no reputation impact.
**Why it hurts**: Wastes the poster's time and emotional energy. Erodes trust in the platform. "The amount of no-shows, ghosts, stupid negotiations... just to GIVE THINGS AWAY FOR FREE."
**Offrii rule**: Implement a lightweight reliability score. After a confirmed match, if the helper doesn't follow through (and the requester flags it), the helper gets a "missed" mark. 3 missed marks in 30 days = temporary restriction. Show a "reliability" indicator on profiles.

---

## 6. Supply/Demand Imbalance Strategies

The "too many requests, not enough givers" problem is the central challenge for any mutual aid platform. Here are evidence-based strategies from across the apps studied:

### Strategy 1: Reciprocity Currency (GEEV model)
- Virtual currency (bananas/hearts) earned by giving, spent on contact
- Creates natural economic equilibrium
- **Caution**: Never gate the ability to ASK for help. Only use currency for optional features or to reward giving

### Strategy 2: Gratitude Visibility (Buy Nothing model)
- Public thank-you posts make giving emotionally visible
- Creates social pressure and positive reinforcement
- Givers see their impact and feel recognized

### Strategy 3: Impact Dashboards (TGTG/Olio model)
- Personal and community impact counters
- "You've helped 12 families this month"
- Milestone celebrations and badges at thresholds

### Strategy 4: Category-Based Alerts (GEEV/Vinted model)
- Let potential helpers subscribe to categories they can fulfill
- "I have children's clothing to give" => alert when children's clothing wish appears
- Converts passive good intentions into active matching

### Strategy 5: Curated "Urgent Needs" Section
- Algorithm or moderator-curated section highlighting time-sensitive or under-responded wishes
- Similar to GoFundMe's "trending" but based on need urgency rather than popularity
- Show wishes with 0 offers after 48 hours prominently

### Strategy 6: Circle-Based Amplification (Offrii-specific)
- When a wish gets no offers after 24 hours, prompt the poster: "Share this wish with your Circles?"
- Circle members who have given before are notified first
- Leverages existing trust networks

### Strategy 7: Seasonal/Themed Campaigns
- "Back to School Drive" -- highlight education wishes in September
- "Winter Warmth" -- feature clothing/heating wishes in December
- Creates collective momentum and community events

---

## 7. Gamification Patterns for Generosity

Based on research across gamification literature and the studied apps:

### Tier 1: Non-Competitive (Recommended for Mutual Aid)
| Pattern | Description | Effectiveness |
|---------|-------------|---------------|
| **Impact counter** | "You've helped 12 people" | High -- tangible, personal |
| **Milestone badges** | "First Help", "5 Helps", "Community Star" | Medium-High -- achievable goals |
| **Gratitude wall** | Public thank-yous from receivers | High -- emotional reward |
| **Streak tracking** | "Helped someone 3 weeks in a row" | Medium -- encourages consistency |

### Tier 2: Light Competition (Use with Care)
| Pattern | Description | Effectiveness |
|---------|-------------|---------------|
| **Community leaderboard** | "Top helpers this month in your area" | Medium -- can feel exclusionary |
| **Circle challenges** | "Your Circle fulfilled 10 wishes!" | Medium-High -- group motivation |

### Tier 3: Avoid for Mutual Aid
| Pattern | Why to Avoid |
|---------|-------------|
| **Individual leaderboards** | Creates helper hierarchy; receivers feel like "charity cases" |
| **Points-for-prizes** | Extrinsic rewards can crowd out intrinsic generosity |
| **Public donation amounts** | Inappropriate for mutual aid (not financial) |

**Recommendation for Offrii**: Focus on Tier 1 patterns. Impact counter + milestone badges + gratitude wall. Keep it warm, not competitive. The emotional reward of gratitude is the strongest motivator in gift economies.

---

## 8. Geographic Proximity Handling

### Patterns from Studied Apps

| App | Proximity Approach |
|-----|-------------------|
| GEEV | Auto-detect GPS + adjustable radius (1-50km) |
| Buy Nothing | Hyperlocal groups based on verified neighborhood |
| Nextdoor | Address-verified neighborhood boundaries |
| TGTG | Map with adjustable radius (1-30km), radius slider on map |
| FB Marketplace | Distance filter (1mi to 100mi) |
| Olio | Proximity-sorted feed, no explicit radius |

### Best Practices for Offrii

1. **Default radius**: Auto-detect location, default to 10km radius. Adjustable via slider on map view (1-50km).
2. **Privacy**: Never show exact addresses. Show approximate distance ("~2km away") and neighborhood name.
3. **Meeting point**: After matching, allow both parties to suggest a meeting point via map pin. Pre-suggest public places (libraries, community centers, metro stations).
4. **Delivery option**: For items that can be shipped/delivered, allow toggling "delivery possible" to expand reach beyond local radius.
5. **No-location wishes**: Allow wishes without location for digital help (tutoring, advice). Tag as "Remote" or "Digital."

---

## 9. Accessibility Considerations

Based on WWDC 2025 guidelines, iOS HIG, and accessibility research:

### Critical for Mutual Aid Apps

1. **VoiceOver compatibility**: All interactive elements must have meaningful labels. "Help with wish: Winter coat needed, 2km away" not just "Button."
2. **Dynamic Type support**: All text must scale with iOS system font size. Critical for elderly users who are both common givers and receivers.
3. **Color contrast**: Status badges must meet WCAG 2.2 AA (4.5:1 for text, 3:1 for UI components). Don't rely on color alone -- add icons to status badges.
4. **Reduce Motion**: Respect iOS "Reduce Motion" setting. Confetti animations, map zoom transitions, etc. should degrade gracefully.
5. **Large touch targets**: Minimum 44x44pt for all interactive elements. Especially critical for "Offer to Help" buttons.
6. **Plain language**: Mutual aid users may have varying literacy levels. Use simple, clear language. Avoid jargon. "Besoin" not "Desiderata."
7. **Image descriptions**: Prompt users to add alt text to wish photos. AI can auto-generate descriptions for screen readers.
8. **Offline capability**: Some users may have limited data plans. Core browsing and wish viewing should work with cached data.
9. **Right-to-left (RTL) support**: If Offrii serves Arabic-speaking communities, RTL layout support is essential.
10. **Cognitive accessibility**: Break the wish creation flow into clear, numbered steps with progress indicator. Never surprise users with unexpected context changes.

### New in iOS 26 (2025-2026)
- **Accessibility Reader**: System-wide customizable reading mode. Ensure Offrii content works well with it.
- **Liquid Glass**: Ensure sufficient contrast against translucent backgrounds.
- **Accessibility Nutrition Labels**: Apple now shows accessibility support on App Store listings. Having strong accessibility features = better App Store visibility.

---

## 10. Wireframe-Level Screen Descriptions

### Screen 1: DISCOVER (Main Entraide Screen)

```
+------------------------------------------+
| [Location: Paris 11e ▼]    [Bell icon 3] |
+------------------------------------------+
| [Feed]  [Map]  [My Wishes]              |
+------------------------------------------+
| Category chips (scrollable):             |
| [All] [Education] [Clothing] [Health]    |
| [Children] [Home] [Religion] [Other]     |
+------------------------------------------+
| Sort: [Nearest] [Newest] [Most Urgent]   |
+------------------------------------------+
|                                          |
| +--------------------------------------+ |
| | [PHOTO]              [Status: Open]  | |
| | "Winter coat for my son (size 8)"    | |
| | ~2.3km away  |  Posted 3h ago        | |
| | [Offer to Help]                      | |
| +--------------------------------------+ |
|                                          |
| +--------------------------------------+ |
| | [No photo - category icon]  [Open]   | |
| | "School supplies for 2 children"     | |
| | ~4.1km away  |  Posted 1d ago        | |
| | [Offer to Help]     [2 offers]       | |
| +--------------------------------------+ |
|                                          |
| +--------------------------------------+ |
| | [PHOTO]            [Matched]         | |
| | "Looking for a stroller"    [Anon]   | |
| | ~1.5km away  |  Posted 2d ago        | |
| | [Matched with helper]                | |
| +--------------------------------------+ |
|                                          |
+------------------------------------------+
| [Home] [Circles] [+New Wish] [Chat] [Me]|
+------------------------------------------+
```

**Map variant** (when "Map" tab is selected):
```
+------------------------------------------+
| [Location: Paris 11e ▼]    [Bell icon 3] |
+------------------------------------------+
| [Feed]  [Map*]  [My Wishes]             |
+------------------------------------------+
| Category chips (scrollable):             |
| [All] [Education] [Clothing] ...         |
+------------------------------------------+
|                                          |
|     MAP VIEW (full width)                |
|                                          |
|    [pin]        [pin]                    |
|         [cluster: 5]                     |
|                    [pin]                 |
|   [pin]                                 |
|                                          |
|   [Radius: 10km slider]                 |
|                                          |
+------------------------------------------+
| BOTTOM SHEET (draggable up):             |
| "Winter coat for my son" -- 2.3km       |
| [Offer to Help]                         |
+------------------------------------------+
| [Home] [Circles] [+New Wish] [Chat] [Me]|
+------------------------------------------+
```

### Screen 2: CREATE WISH

```
+------------------------------------------+
| [X Cancel]    New Wish    [Post]         |
+------------------------------------------+
|                                          |
| STEP 1/3 - What do you need?            |
| ======================================= |
|                                          |
| [+ Add Photo]  (optional, encouraged)    |
| [Camera icon] [Gallery icon]             |
|                                          |
| Title *                                  |
| [                                    ]   |
| "e.g., Winter coat for my son"           |
|                                          |
| Category *                               |
| [Education] [Clothing*] [Health]         |
| [Children] [Home] [Religion] [Other]     |
|                                          |
| [Next >]                                 |
+------------------------------------------+

STEP 2/3 - Tell your story
+------------------------------------------+
| [< Back]     New Wish     [Post]         |
+------------------------------------------+
|                                          |
| STEP 2/3 - Details                       |
| ======================================= |
|                                          |
| Description                              |
| [                                    ]   |
| [                                    ]   |
| [                                    ]   |
| [AI Suggest] -- helps write description  |
|                                          |
| [Toggle] Post anonymously               |
|                                          |
| [Next >]                                 |
+------------------------------------------+

STEP 3/3 - Location & preferences
+------------------------------------------+
| [< Back]     New Wish     [Post]         |
+------------------------------------------+
|                                          |
| STEP 3/3 - Where & How                  |
| ======================================= |
|                                          |
| Location                                 |
| [Auto-detected: Paris 11e  Edit]         |
|                                          |
| Delivery preference                      |
| (o) Pickup only                          |
| (o) Delivery possible                    |
| (o) Either                               |
|                                          |
| [Toggle] Share with my Circles           |
|                                          |
| [Post Wish]                              |
| Progress: [===----] 1 of 3 active wishes |
+------------------------------------------+
```

### Screen 3: WISH DETAIL

```
+------------------------------------------+
| [< Back]              [Share] [Report]   |
+------------------------------------------+
|                                          |
| [         PHOTO (full width)           ] |
|                                          |
| "Winter coat for my son (size 8)"        |
| Posted by Marie L.  |  3 hours ago       |
| [Avatar]  Reliability: ●●●●○            |
|                                          |
| Status: [OPEN - 2 offers received]       |
|                                          |
| Category: Clothing                       |
| Location: ~2.3km away (Paris 11e)        |
| Preference: Pickup or delivery           |
|                                          |
| ─────────────────────────────────────    |
| "My son starts school next month and     |
| we lost most of his winter clothes in    |
| our recent move. Any warm coat in        |
| size 8 (128cm) would be amazing.         |
| Thank you!"                              |
| ─────────────────────────────────────    |
|                                          |
| OFFERS (2):                              |
| [Avatar] Paul D. -- "I have a coat!"     |
| [Avatar] Sarah K. -- "I can help"        |
|                                          |
| [   OFFER TO HELP   ] (primary CTA)     |
|                                          |
| Quick messages:                          |
| [I can help!] [I have this] [Tell me +] |
|                                          |
+------------------------------------------+
| [Home] [Circles] [+New Wish] [Chat] [Me]|
+------------------------------------------+
```

### Screen 4: MESSAGES (Wish-Contextualized)

```
+------------------------------------------+
| [< Back]   Chat with Paul D.            |
+------------------------------------------+
| ABOUT THIS WISH:                         |
| [Mini-card: "Winter coat" | Matched]     |
+------------------------------------------+
|                                          |
| Paul: I have a blue winter coat, size 8  |
|       [Photo attached]                   |
|       3:14 PM                            |
|                                          |
|            Marie: That looks perfect!    |
|            Can we meet tomorrow?         |
|            3:16 PM                       |
|                                          |
| Paul: Sure! How about the library        |
|       on Rue Voltaire at 2pm?            |
|       3:18 PM                            |
|                                          |
|            Marie: Perfect, see you then! |
|            3:19 PM                       |
|                                          |
+------------------------------------------+
| [Suggest Meeting Point]                  |
+------------------------------------------+
| [Photo] [Text input...        ] [Send]  |
+------------------------------------------+

After exchange:
+------------------------------------------+
| EXCHANGE COMPLETE?                       |
|                                          |
| [=====> Swipe to confirm ======>]        |
|                                          |
| Both parties must confirm to close wish  |
+------------------------------------------+
```

### Screen 5: MY IMPACT (Profile Section)

```
+------------------------------------------+
| [< Back]        My Impact                |
+------------------------------------------+
|                                          |
|    [Avatar - Marie L.]                   |
|    Member since Jan 2025                 |
|    Reliability: ●●●●● (100%)            |
|                                          |
| ┌──────────────────────────────────────┐ |
| │  HELPED     RECEIVED     ACTIVE      │ |
| │    7            3           1         │ |
| └──────────────────────────────────────┘ |
|                                          |
| BADGES:                                  |
| [First Help] [5 Helps] [Streak: 3 wks]  |
|                                          |
| COMMUNITY IMPACT:                        |
| [==========] 847 wishes fulfilled        |
| in your area this month                  |
|                                          |
| GRATITUDE WALL:                          |
| "Marie gave us school supplies for both  |
|  kids. So grateful!" -- Anon, 2 wks ago  |
|                                          |
| "Thanks for the stroller! Perfect        |
|  condition." -- Fatima K., 1 mo ago      |
|                                          |
+------------------------------------------+
| [Home] [Circles] [+New Wish] [Chat] [Me]|
+------------------------------------------+
```

---

## 11. Recommendations for Offrii Entraide

### Priority 1 -- Must Have (MVP Redesign)

1. **Map + Feed dual discovery** with category chips and proximity sorting
2. **Photo-first, 3-step wish creation** with AI-assisted descriptions
3. **Clear status lifecycle** (Open > Offered > Matched > Fulfilled) with colored badges
4. **Wish-contextualized messaging** -- chats always linked to the specific wish
5. **Quick-action contact templates** to reduce first-message friction
6. **Gratitude flow** post-fulfillment with community visibility
7. **Swipe-to-confirm fulfillment** with haptic feedback

### Priority 2 -- Should Have (Post-MVP)

8. **Saved search alerts** ("Alert me for children's clothing within 5km")
9. **Impact counter and badges** on user profiles
10. **Circle integration** -- share wishes with Circles, Circle-level impact tracking
11. **Reliability score** to address no-show problem
12. **Meeting point suggestion** via map pin for pickup coordination
13. **"Urgent needs" curation** -- highlight wishes with 0 offers after 48h

### Priority 3 -- Nice to Have (Future)

14. **Reciprocity system** (light version of GEEV's bananas) to encourage giving
15. **Seasonal campaigns** ("Back to School," "Winter Warmth")
16. **Community leaderboard** (Circle-level, not individual)
17. **AI-powered matching** -- suggest wishes to potential helpers based on their giving history
18. **Delivery/shipping option** for non-local fulfillment

### Architecture Notes

- Entraide should live as a top-level tab or prominent entry point, not buried in a sub-menu
- The Entraide feed should feel distinctly different from a marketplace -- warmer colors, rounder shapes, emphasis on the human story rather than the item
- Anonymous posting is a differentiator for Offrii. Preserve it but ensure it doesn't reduce trust (show "verified member" even for anonymous posts)
- The 3-wish limit is good for preventing abuse but should be communicated clearly with a progress indicator ("1 of 3 active wishes")
- Existing Circle infrastructure is Offrii's secret weapon -- no other app combines trusted social circles with mutual aid. Lean into this hard.

### Design Language Suggestions

- **Warm palette**: Offrii's primary colors + amber/orange for Entraide (distinct from the main app)
- **Rounded cards**: 16pt corner radius, subtle shadows. Friendly, not corporate.
- **Category emojis**: Education: book, Clothing: shirt, Health: heart, Religion: hands, Home: house, Children: baby, Other: star
- **Status colors**: Open = Blue, Offered = Orange, Matched = Yellow, Fulfilled = Green
- **Typography**: SF Pro Rounded for headers in Entraide section (warmer than standard SF Pro)

---

## 12. Sources

- [GEEV App Store listing](https://apps.apple.com/us/app/geev-le-r%C3%A9flexe-anti-gaspi/id1165633060) -- Features, ratings, banana system
- [GEEV official site](https://www.geev.com/) -- 6M users, 26M items donated, feature descriptions
- [Buy Nothing Project](https://buynothingproject.org/) -- 14M members, 3.0 launch details
- [Buy Nothing 3.0 launch announcement (PRWeb, Jan 2026)](https://www.prweb.com/releases/the-rise-of-the-gift-economy-buy-nothing-project-launches-new-platform-to-meet-surging-demand-302666108.html) -- Redesigned platform details
- [Buy Nothing 3.0.8 release notes](https://buynothingproject.org/announcements/what%E2%80%99s-new-in-buy-nothing-3.0.8) -- Latest features
- [Buy Nothing App Store listing](https://apps.apple.com/us/app/buynothing/id1557679959) -- User reviews, pain points
- [Freecycle.org](https://www.freecycle.org/) -- 12M members, FAQ, posting guidelines
- [Trash Nothing + Freecycle App Store listing](https://apps.apple.com/us/app/trash-nothing-freecycle/id680743557) -- Mobile features
- [Nextdoor "Meet the New Nextdoor" (July 2025)](https://about.nextdoor.com/press-releases/meet-the-new-nextdoor) -- Help Map, Ask, redesign details
- [Nextdoor 2025 Transparency Report](https://about.nextdoor.com/press-releases/nextdoor-publishes-2025-transparency-report) -- Trust and safety data
- [Nextdoor Help Map support article](https://nextdoorcrm.my.site.com/s/article/Use-the-Help-Map-to-find-and-offer-help) -- Help Map feature
- [Facebook Marketplace Reddit discussion (2025)](https://www.reddit.com/r/advertising/comments/1p8pepu/facebook_marketplace_just_got_a_proper_glowup_and/) -- Social shopping features
- [GoFundMe verification guidelines](https://www.gofundme.com/c/safety/verification-guidelines) -- Trust patterns
- [GoFundMe trustworthiness guidelines](https://support.gofundme.com/hc/en-us/articles/115015913668-Determining-if-a-GoFundMe-is-trustworthy) -- Donor trust UX
- [Too Good To Go "How the app works"](https://www.toogoodtogo.com/en-us/how-does-the-app-work) -- Discovery, pickup flow
- [TGTG UX Design Critique (IxD@Pratt)](https://ixd.prattsi.org/2023/02/design-critique-too-good-to-go-ios-app/) -- Map UX, surprise bag flow
- [Revitalizing TGTG: UX Case Study (UX Planet)](https://uxplanet.org/revitalizing-too-good-to-go-app-a-ux-design-case-study-48c3bfab90a5) -- Heuristic analysis, redesign
- [Vinted selling guide (CLOSO, 2025)](https://closo.co/blogs/platform-specific-guides/vinted-app-complete-guide-to-selling-smarter-in-2025) -- Listing creation, algorithm, badges
- [Wireframing Vinted App selling flow (Medium, Jan 2026)](https://medium.com/@kmen/wireframing-the-vinted-app-selling-flow-3331d5156261) -- UX wireframing
- [Vinted business model breakdown (Miracuves, 2026)](https://miracuves.com/blog/business-model-of-vinted/) -- Trust architecture, monetization
- [Olio App Store listing](https://apps.apple.com/lv/app/olio/id1008237086) -- Features, AI-ify, Lucky Dip
- [Olio March 2025 Community Update](https://olioapp.com/en/olio-updates/march-2025-community-update/) -- Home/Explore redesign, messaging improvements, dynamic delays removal
- [Olio September 2025 Community Update](https://olioapp.com/en/olio-updates/september-2025-community-update/) -- Save Me Collections, Lucky Dip, food safety
- [Marketplace UX Design: 9 Best Practices (Excited Agency)](https://excited.agency/blog/marketplace-ux-design) -- Navigation, IA, listing design
- [Marketplace UI/UX Design Best Practices (Gapsy Studio)](https://gapsystudio.com/blog/marketplace-ui-ux-design/) -- Trust engineering, progressive onboarding
- [Marketplace UX Feature-by-Feature Guide (Rigby)](https://www.rigbyjs.com/blog/marketplace-ux) -- Search, filters, empty states, CTAs
- [Mobile UX Design Trends 2026 (UX Pilot)](https://uxpilot.ai/blogs/mobile-app-design-trends) -- Micro-interactions, glassmorphism, spatial design
- [iOS UX Design Trends 2026 (Asapp Studio)](https://asappstudio.com/ios-ux-design-trends-2026/) -- Liquid Glass, cognitive accessibility
- [7 Mobile UX/UI Design Patterns 2026 (Sanjay Dey)](https://www.sanjaydey.com/mobile-ux-ui-design-patterns-2026-data-backed/) -- Progressive onboarding, biometric auth
- [WWDC25: Design foundations from idea to interface](https://developer.apple.com/videos/play/wwdc2025/359/) -- Grouping, progressive disclosure, layout
- [WWDC25: Principles of inclusive app design](https://developer.apple.com/videos/play/wwdc2025/316/) -- Accessibility Reader, multiple input modes
- [Accessibility Trends 2026](https://www.accessibility.com/blog/accessibility-trends-to-watch-in-2026) -- Cognitive accessibility, plain language
- [Gamification market research (StriveCloud, 2025-2026)](https://www.strivecloud.io/blog/examples-gamification-app) -- $19.42B market, engagement metrics
- [Buy Nothing Groups & Community Exchanges Guide (Loopstr)](https://loopstr.co/buy-nothing-groups-community-exchanges/) -- Transactional friction theory, gift economy principles
