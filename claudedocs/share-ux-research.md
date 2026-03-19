# Share Flow UX Research: Wishlist & Gift Registry Apps

**Date**: 2026-03-17
**Confidence**: 0.85 (high -- primary sources verified across multiple platforms)
**Scope**: Amazon, Pinterest, Elfster/Giftster/MyRegistry, Notion, Linear, Apple iOS Share Sheet, Google Shopping

---

## Executive Summary

Sharing UX in wishlist/gift apps follows a consistent two-tier pattern: **private sharing to specific people/groups** and **public link sharing** for broader distribution. The best implementations keep these two modes visually separated but accessible from the same entry point. Offrii's current `ShareToCircleSheet` handles the "private circles" half well; the missing piece is the **public link** half plus a unified entry point that frames both options clearly.

---

## 1. Platform-by-Platform Analysis

### 1.1 Amazon Wish List

**Share entry point**: "Send list to others" link at the top of any list, or "Invite" button on mobile.

**Share flow**:
1. Tap "Invite" or "Send list to others"
2. A modal appears with **two clear permission modes as the first choice**:
   - **View Only** -- anyone with the link can see items but cannot edit
   - **View and Edit** -- anyone with the link can see, add, modify, and delete items (requires a public Amazon profile)
3. After selecting a mode, two distribution methods appear:
   - **Copy Link** -- copies URL to clipboard
   - **Invite by email** -- opens email compose with prefilled link

**Key UX patterns**:
- Permission level is selected BEFORE the distribution method (mode-first, channel-second)
- No link expiration -- Amazon links are permanent until the list privacy is changed
- No granular per-person permissions; the link itself carries the permission level
- The list privacy setting (Private / Shared / Public) automatically changes from Private to Shared when you generate a share link
- **Manage access**: List owner can "Manage people" from the "..." menu to see all invitees and remove access
- **No claim/reserve system** built into core wishlists (third-party workarounds exist)

**Strengths**: Dead simple. Two choices then two actions. No cognitive overload.
**Weaknesses**: No link expiration. No reservation/claim system. Privacy settings are confusing (Private vs Shared vs Public is three states with overlapping meaning).

---

### 1.2 Pinterest Boards

**Share entry point**: "+" icon next to profile picture on desktop, "Invite" button on mobile, or the share arrow button on any board.

**Two distinct sharing mechanisms**:

1. **Collaborator Invitations (private, person-specific)**:
   - Enter email or Pinterest username
   - Choose permission level per collaborator:
     - **Do (almost) everything** -- add, move, delete Pins and sections, comment and react
     - **Save and comment** -- save and organize Pins, comment and react
     - **Invite other people** -- can invite others to the group board
   - Board Requests toggle: allow people to request to join

2. **Link Sharing (public)**:
   - Click the "Share" arrow button
   - Quick-share buttons for major platforms (Messages, WhatsApp, Facebook, X, email)
   - Copy direct link option
   - This is a READ-ONLY share -- link recipients can view but cannot add Pins

**Secret (Private) Boards**:
- Toggle "Keep this board secret" in board settings
- Secret boards are invisible in search, on profile, and in feeds
- You can still invite collaborators to secret boards (same flow as above)
- Making a secret board public is a one-toggle action (and reversible)

**Key UX patterns**:
- Clear separation: "Invite collaborators" (specific people with roles) vs "Share link" (public, read-only)
- Granular permissions per collaborator, not per link
- No link expiration
- Board owner manages collaborators from a visible collaborator list (profile pictures shown on the board)
- **Board requests**: People can request to join, owner approves/denies

**Strengths**: Best-in-class permission granularity per person. Visible collaborator avatars create social proof.
**Weaknesses**: No link expiration. No "view only" collaborator role (you must share via link for that).

---

### 1.3 Elfster

**Share flow**:
1. Wishlist must have items (empty lists cannot be shared)
2. Set Wishlist Privacy:
   - **Public** -- anyone with an Elfster account OR anyone with the link can see it
   - **Limited** -- choose specific groups or individuals who can see it
   - **Private** -- only you can view it
3. Click "Share Wishlist" at the top of the list
4. Copy the link to paste via email, text, or social media

**Key constraint**: A wishlist MUST be set to Public to share outside of Elfster. Limited visibility only works for Elfster users within groups.

**Claim/Reserve pattern**:
- Other users can "mark that I bought an item" on someone else's wishlist
- The wishlist owner does NOT see who reserved what (preserves surprise)
- Duplicate purchase prevention is the core value proposition

**Strengths**: Privacy levels are explicit and clearly named. Group-based sharing maps well to real-world gift exchange contexts.
**Weaknesses**: Forcing "Public" for external sharing is confusing -- users want to share a link privately, not make the list discoverable. No link expiration.

---

### 1.4 Giftster

**Share flow (two distinct paths)**:

**Path A: Family/Friends (ongoing)**
- Start or join a Giftster "group"
- All group members can see each other's lists
- This is the primary intended usage -- "checking Giftster" becomes a habit before birthdays/holidays

**Path B: Registry (one-time, public)**
- Set list privacy to **public**
- Share via link, email, or QR code
- QR code generation is available on web (not yet in mobile apps)
- Guests can shop the list without creating an account

**Gift reservation system**:
- Family members can privately reserve items
- Purchases are hidden from the list maker
- "Mark as bought" status visible to other shoppers but NOT to the list owner

**Key UX patterns**:
- Clear separation between "family group" sharing (ongoing, private) and "registry" sharing (one-time, public link)
- QR code support is a standout feature for physical invitations (weddings, showers)
- Mailing address can be displayed to guest shoppers if the owner opts in

**Strengths**: The group model maps perfectly to families. QR code sharing bridges physical/digital.
**Weaknesses**: Extremely outdated UI. Confusing setup flow. Poor mobile experience.

---

### 1.5 Giftwhale (Notable Newcomer)

**Gift reservation flow** (best-in-class in this category):
1. Find a wish list (via shared link or browsing friends)
2. Click "Reserve" on any item
3. Others see it's reserved; the recipient does NOT
4. Buy the item from the linked store
5. Mark as "purchased" in your dashboard
6. **Guest reservations**: No account needed to reserve

**Key insight**: Giftwhale proves that guest reservations (no login required) dramatically reduce friction for the gift-giver side. This is the pattern Offrii should consider for public link recipients.

---

### 1.6 Notion

**Share entry point**: "Share" button at the top-right of every page.

**Share modal structure** (top to bottom):
1. **People section** -- invite specific people by name or email
   - Each person gets a permission dropdown: Full Access / Can Edit / Can Comment / Can View / No Access
   - Can invite workspace members or external guests
2. **General access section** -- three options:
   - **Only people invited** -- most restrictive
   - **Everyone at {workspace}** -- visible to all workspace members (with optional "Hide in search" toggle)
   - **Anyone on the web with link** -- public, even non-Notion users can view
3. **Copy link** button -- always visible at the bottom

**Permission hierarchy**:
- Full Access > Can Edit > Can Edit Content (databases only) > Can Comment > Can View > No Access
- Sub-pages inherit parent permissions by default
- Permissions can be increased but not decreased on sub-pages (key limitation)
- "Highest permission wins" -- if a user has access via multiple routes, they get the most permissive

**Link expiration**: Not natively supported (Figma has it on Enterprise; Notion does not).

**Manage access UI**:
- All invited users appear in the Share modal with their permission level and avatar
- Hover over an avatar at the top-right of any page to see name, email, and last visit time
- Owner can change permission level or remove access inline from the Share modal

**Key UX patterns**:
- **Everything in one modal**: Person-level sharing, general access, and link copying are all in a single, compact share dialog
- The modal is the "manage access" UI -- no separate management screen
- Excellent progressive disclosure: simple by default, powerful on inspection
- Clear visual distinction between "specific people" (top) and "general access" (bottom)

**Strengths**: Single unified share modal that serves as both "share" and "manage" UI. Best progressive disclosure in any productivity app.
**Weaknesses**: No link expiration. No notification when permissions change. Sub-page inheritance is confusing.

---

### 1.7 Linear

**Current sharing model**: Workspace-internal only. No public link sharing.

**Private teams**:
- Issues in private teams are invisible to non-members
- Sharing an issue with a non-member creates a limited view (they can see the issue + sub-issues, comment, change basic properties)
- Shared issues appear under "My Issues > Shared" tab

**Guest role** (Business/Enterprise only):
- Restricted to specified teams
- Cannot access workspace-level admin
- Cannot share issues with others

**Key insight**: Linear explicitly does NOT support public link sharing (this is a frequently requested feature -- GitHub issue #653). Their philosophy is that project management data should stay internal. For Offrii, this validates that having BOTH private and public modes is a competitive advantage.

**Permission model**:
- Workspace Owner > Admin > Member > Guest
- Team-level: Team Owner > Team Member
- Configurable per-team: who can manage members, issues, settings
- No per-item permission granularity (unlike Notion)

---

## 2. iOS Share Sheet Best Practices (Apple HIG)

### 2.1 Technical Implementation

**SwiftUI**: Use `ShareLink` view (preferred) or `UIActivityViewController` (UIKit).

```swift
// SwiftUI - simple
ShareLink(item: URL(string: "https://offrii.com/wish/abc123")!)

// SwiftUI - with preview
ShareLink(
    item: wishItem,
    preview: SharePreview(wishItem.title, image: wishItem.image)
)
```

**Key requirements**:
- Always present on the main thread
- On iPad, MUST specify `popoverPresentationController.sourceView` or the app crashes
- Prepare share data asynchronously BEFORE presenting the sheet (avoid 5-second delays)

### 2.2 Share Sheet Anatomy (iOS 16+)

The share sheet has three sections, top to bottom:
1. **Preview header** -- shows what's being shared (title, image, URL via LinkPresentation/LPLinkMetadata)
2. **People suggestions row** -- recent contacts and conversation threads (configurable; can be hidden with `excludedActivitySectionTypes = .peopleSuggestions`)
3. **App/action grid** -- Messages, Mail, AirDrop, third-party apps, then actions (Copy, Add to Reading List, etc.)

### 2.3 Collaboration vs Copy Modes (iOS 16+)

Apple introduced a **sharing mode popup** in the share sheet:
- **Collaborate** -- for real-time collaboration (CloudKit, iCloud)
- **Send a Copy** -- traditional one-way sharing

You can restrict to only one mode or show both. For Offrii, the share sheet should only offer "Send a Copy" (the link) since real-time collaboration isn't the use case.

### 2.4 Best Practices from Apple HIG + Research

1. **Rich link previews**: Use `LPLinkMetadata` so shared links show a title, subtitle, and thumbnail in Messages/Mail. This dramatically increases tap-through rates.
2. **Don't overload the share sheet**: The system share sheet is for DISTRIBUTION (sending the link somewhere). Permission controls and link configuration should happen BEFORE the share sheet opens.
3. **Pre-share configuration pattern**: Show your own sheet/modal for setting permissions and link options, with a "Share" button that THEN triggers the system share sheet.
4. **Specify recipients when known**: In iOS 18+, you can pre-fill recipients using `INPerson`.

---

## 3. Cross-Platform Pattern Synthesis

### 3.1 The Universal Share Flow Structure

Every well-designed share flow follows this 3-step structure:

```
[1. CONFIGURE]  -->  [2. DISTRIBUTE]  -->  [3. MANAGE]
 What & how         Where/to whom         Review & revoke
```

**Step 1 - Configure** (your app's UI):
- Select permission level (view only / can claim / can edit)
- Set link expiration (if applicable)
- Choose sharing mode (private to people OR public link)

**Step 2 - Distribute** (system + your UI):
- Private: Select circles/people from your app
- Public: Copy link, or trigger system share sheet for Messages/WhatsApp/etc.

**Step 3 - Manage** (your app's UI):
- See who has access
- See active links
- Revoke access or deactivate links
- View link activity (clicks, claims)

### 3.2 "Private Sharing" vs "Public Link" -- How Apps Separate Them

| App | Private Sharing | Public Link | Separation UX |
|-----|----------------|-------------|---------------|
| Amazon | N/A (no groups) | "Copy Link" with permission toggle | Single modal, permission-first |
| Pinterest | "Invite collaborators" (by name/email) | "Share" arrow (read-only link) | Two separate entry points |
| Elfster | "Limited" privacy (select groups) | "Public" privacy + copy link | Privacy setting gates the flow |
| Giftster | "Join group" (ongoing) | "Public" list + link/QR | Two completely different paths |
| Notion | "Add people" (top of modal) | "Anyone on web" toggle (bottom) | **Single modal, two sections** |
| Linear | Share issue (internal only) | N/A (not supported) | N/A |

**Winner for Offrii's use case: Notion's approach** -- a single share modal with clear visual separation between "share with specific circles" (top) and "public link" (bottom).

### 3.3 Link Expiration Patterns

Most wishlist/gift apps do NOT implement link expiration. This is borrowed from enterprise/productivity tools:

| Platform | Expiration Options | UX Pattern |
|----------|-------------------|------------|
| Amazon Wish List | None | -- |
| Pinterest | None | -- |
| Elfster/Giftster | None | -- |
| Notion | None (natively) | -- |
| Figma (Enterprise) | 1 hour to 1 year (freeform date picker) | Checkbox "Link expiration" + date picker |
| SharePoint/OneDrive | Admin-set max + user-adjustable | Date picker with admin ceiling |
| Rebrandly | Specific date/time | Date + time picker |

**Recommendation for Offrii**: Use **preset durations** rather than a date picker. Wishlist sharing is casual, not enterprise. Suggested presets:
- **1 day** (flash share for an event)
- **1 week** (default, good for birthdays)
- **1 month** (good for holidays)
- **Never** (permanent link)

This avoids the cognitive load of a date picker for a use case where exact dates rarely matter.

### 3.4 Permission Models for Gift/Wishlist Context

| Permission Level | Who Needs It | Pattern |
|-----------------|-------------|---------|
| **View only** | Casual browsers, "inspiration" sharing | Default for public links |
| **Can claim/reserve** | Gift givers who want to prevent duplicates | Default for circle members |
| **Can edit** | Co-owners of shared lists (rare in gift context) | Only for circle admins/co-creators |

**Critical UX insight from Giftwhale**: The claim/reserve action should be HIDDEN from the list owner. This preserves the gift surprise. The owner sees the item as normal; everyone else sees "Reserved by [name]" or just "Reserved."

**Guest reservations** (no account required): Giftwhale proves this works. For public link recipients, allow claiming without sign-up -- just ask for a name. This is a significant competitive advantage.

### 3.5 "Manage Existing Shares" UI Patterns

**Pattern A: Inline in Share Modal (Notion)**
- The share modal IS the management UI
- Invited people appear as a list with permission dropdowns
- General access toggle visible below
- Single entry point for both sharing and management

**Pattern B: Separate Management Screen (Amazon)**
- "Manage people" accessible from a "..." overflow menu
- Shows list of invitees with names
- Separate from the initial share flow
- Less discoverable but cleaner initial experience

**Pattern C: Visible Avatars (Pinterest)**
- Collaborator avatars displayed on the board itself
- Tapping avatars opens management
- Most visually integrated but less scalable

**Recommendation for Offrii**: Pattern A (Notion-style) is best. The share sheet should double as the manage-access view. Show:
- Circles already shared with (with checkmark, tappable to unshare)
- Public link section with active/inactive status and copy button
- This is already partially implemented in `ShareToCircleSheet.swift` for circles

### 3.6 Link Preview / Copy UX

**Best practices observed**:
1. **Show the link visually before copying**: Display a truncated URL or a styled link card (title + thumbnail) so users know what they're sharing
2. **"Copied!" feedback**: Brief inline confirmation (checkmark replacing copy icon, or toast). Amazon and Notion both use this.
3. **Rich link metadata**: When shared via Messages/social, the link should unfurl into a card with:
   - Item image (or list cover image)
   - Item title or list name
   - "View on Offrii" subtitle
   - Offrii branding
4. **QR code** (Giftster pattern): Valuable for physical events (baby showers, weddings, birthday parties). Generate a QR code alongside the link.

---

## 4. Anti-Patterns to Avoid

### 4.1 Sharing-Specific Anti-Patterns

1. **Forcing account creation to view shared content**: Elfster requires an Elfster account to view even "Public" lists in some flows. This kills share conversion. Public links should be viewable without any login.

2. **Conflating privacy settings with share actions**: Amazon's three-state system (Private / Shared / Public) confuses users because sharing a link automatically changes the list from Private to Shared. The privacy state should be a CONSEQUENCE of sharing actions, not a prerequisite.

3. **No visual distinction between "already shared" and "not yet shared"**: Users need to immediately see the current sharing state. Offrii's current implementation handles this well with checkmarks vs empty circles.

4. **Hiding share management**: If users can't easily find WHERE something is shared and revoke access, they won't share in the first place. The "manage" UI must be as discoverable as the "share" UI.

5. **Requiring the full system share sheet for simple link copies**: The system share sheet is overkill when users just want to copy a link. Always provide a "Copy Link" button BEFORE/alongside the share sheet trigger.

6. **No confirmation of successful sharing**: After sharing to circles or generating a link, provide clear feedback (haptic + visual). Offrii already does this with `OffriiHaptics.success()`.

7. **Permission confusion through vague labels**: "View and Edit" (Amazon) is unclear about WHAT can be edited. Use domain-specific language: "Can claim gifts" is better than "Can edit" in a wishlist context.

### 4.2 General Mobile UX Anti-Patterns Relevant to Sharing

1. **Requesting permissions before demonstrating value**: Don't ask for contacts access the first time someone opens the share flow. Wait until they tap "Invite from contacts."

2. **Too many taps to share**: The share action should be reachable in 2 taps maximum from any item. Offrii's current flow (item > share button > circle sheet) is 2 taps -- good.

3. **Modal fatigue**: Avoid stacking sheets. If the share sheet opens another sheet (e.g., "create circle first"), users lose context. Use inline flows or progressive disclosure instead.

---

## 5. Recommendations for Offrii

### 5.1 Unified Share Sheet Design

Replace the current circles-only `ShareToCircleSheet` with a **unified share modal** that has two clearly separated sections:

```
+------------------------------------------+
|          Share "[Item Name]"             |
+------------------------------------------+
|                                          |
|  SHARE TO CIRCLES                        |
|  [Circle avatars with check/uncheck]     |
|  [Already shared circles highlighted]    |
|                                          |
|  ----------------------------------------|
|                                          |
|  SHARE VIA LINK                          |
|  [Toggle: Link active / inactive]        |
|  [Permission: View only | Can claim]     |
|  [Expires: 1 day | 1 week | 1 month |   |
|            Never]                        |
|  [Copy Link button]                      |
|  [Share...  (system share sheet)]        |
|  [QR Code button]                        |
|                                          |
+------------------------------------------+
|        [Apply Changes]                   |
+------------------------------------------+
```

### 5.2 Permission Model

| Audience | Default Permission | Options |
|----------|-------------------|---------|
| Circle members | Can claim/reserve | View only, Can claim |
| Public link viewers | View only | View only, Can claim |
| List owner | Full control | (always) |

**Claim behavior**:
- Circle members: Claim is visible to other circle members, HIDDEN from list owner
- Public link: Guest claim (name only, no account) visible to other viewers, hidden from owner
- Claimed items show a subtle "Reserved" badge to non-owners

### 5.3 Link Expiration

- Default: **1 week**
- Presets: 1 day / 1 week / 1 month / Never
- Use a **segmented control** or **chip selector**, NOT a date picker
- When a link expires, it shows a "This link has expired" page with Offrii branding and a CTA to download the app

### 5.4 Link Preview (OpenGraph)

Ensure the public share page serves proper OpenGraph meta tags:
```html
<meta property="og:title" content="Yassine's Birthday Wishlist" />
<meta property="og:description" content="3 items - View and claim gifts on Offrii" />
<meta property="og:image" content="[item collage or first item image]" />
<meta property="og:url" content="https://offrii.com/share/abc123" />
```

This ensures rich previews in Messages, WhatsApp, Slack, etc.

### 5.5 Implementation Priority

1. **P0**: Add "Share via Link" section below the existing circle sharing in `ShareToCircleSheet`
2. **P0**: Public link generation with copy button + system share sheet trigger
3. **P1**: Link expiration with preset durations
4. **P1**: Guest claim/reserve for public link viewers (no account required)
5. **P2**: QR code generation for physical sharing
6. **P2**: Link analytics (view count, claim count)
7. **P3**: OpenGraph rich previews for the share page

---

## 6. Sources

- Amazon Wish List sharing: amazon.com.au/gp/help, businessinsider.com, aboutamazon.co.uk
- Pinterest board sharing: help.pinterest.com, tailwindapp.com/blog
- Elfster wishlists: help.elfster.com
- Giftster sharing: help.giftster.com
- Giftwhale reservations: giftwhale.com/features/gift-reservations
- Notion sharing: notion.com/help, thomasjfrank.com, notion.vip
- Linear permissions: linear.app/docs, github.com/linear/linear/issues/653
- Apple HIG / Share Sheet: developer.apple.com/documentation/uikit, developer.apple.com/design/human-interface-guidelines
- Figma link expiration: help.figma.com
- SharePoint/OneDrive expiration: office365itpros.com, sharepointmaven.com
- Nielsen Norman Group (permission UX): nngroup.com/articles/permission-requests, nngroup.com/articles/wishlists-gift-certificates
- UX anti-patterns: uxplanet.org, jakobnielsenphd.substack.com
