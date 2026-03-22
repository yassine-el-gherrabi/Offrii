import SwiftUI

// MARK: - CircleActivityFeed

struct CircleActivityFeed: View {
    let events: [CircleEventResponse]
    let currentUserId: UUID?

    private enum TimeGroup: String {
        case today
        case yesterday
        case thisWeek
        case older

        var label: String {
            switch self {
            case .today:    return NSLocalizedString("activity.today", comment: "Today")
            case .yesterday: return NSLocalizedString("activity.yesterday", comment: "Yesterday")
            case .thisWeek: return NSLocalizedString("activity.thisWeek", comment: "This week")
            case .older:    return NSLocalizedString("activity.older", comment: "Older")
            }
        }
    }

    private var groupedEvents: [(group: TimeGroup, events: [CircleEventResponse])] {
        let calendar = Calendar.current
        let now = Date()

        var groups: [TimeGroup: [CircleEventResponse]] = [:]

        for event in events {
            let group: TimeGroup
            if calendar.isDateInToday(event.createdAt) {
                group = .today
            } else if calendar.isDateInYesterday(event.createdAt) {
                group = .yesterday
            } else if let weekAgo = calendar.date(byAdding: .day, value: -7, to: now),
                      event.createdAt >= weekAgo {
                group = .thisWeek
            } else {
                group = .older
            }
            groups[group, default: []].append(event)
        }

        let order: [TimeGroup] = [.today, .yesterday, .thisWeek, .older]
        return order.compactMap { group in
            guard let evts = groups[group], !evts.isEmpty else { return nil }
            return (group: group, events: evts)
        }
    }

    var body: some View {
        if events.isEmpty {
            Spacer()
            OffriiEmptyState(
                icon: "bell.slash",
                title: NSLocalizedString("circles.detail.noActivity", comment: ""),
                subtitle: NSLocalizedString("circles.detail.noActivitySubtitle", comment: "")
            )
            Spacer()
        } else {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                    ForEach(groupedEvents, id: \.group.rawValue) { section in
                        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                            Text(section.group.label)
                                .font(OffriiTypography.footnote)
                                .fontWeight(.semibold)
                                .foregroundColor(OffriiTheme.textMuted)
                                .textCase(.uppercase)

                            ForEach(section.events) { event in
                                eventRow(event)
                            }
                        }
                    }
                }
                .padding(OffriiTheme.spacingBase)
            }
        }
    }

    // MARK: - Event Row

    @ViewBuilder
    private func eventRow(_ event: CircleEventResponse) -> some View {
        HStack(alignment: .top, spacing: OffriiTheme.spacingSM) {
            // Avatar
            AvatarView(event.actorUsername, size: .small)

            // Icon + text
            VStack(alignment: .leading, spacing: 2) {
                Text(descriptionForEvent(event))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.text)

                Text(event.createdAt, style: .relative)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }

            Spacer()

            Image(systemName: iconForEvent(event.eventType))
                .font(.system(size: 14))
                .foregroundColor(colorForEvent(event.eventType))
                .frame(width: 24)
        }
        .padding(OffriiTheme.spacingSM)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusSM)
    }

    // MARK: - Helpers

    private func iconForEvent(_ type: String) -> String {
        switch type {
        case "item_shared":   return "square.and.arrow.up"
        case "item_claimed":  return "gift.fill"
        case "item_unclaimed": return "gift"
        case "item_received": return "checkmark.circle.fill"
        case "member_joined": return "person.badge.plus"
        case "member_left":          return "person.badge.minus"
        case "share_rule_set":       return "square.and.arrow.up.on.square"
        case "share_rule_removed":   return "square.and.arrow.up.on.square.fill"
        default:                     return "bell.fill"
        }
    }

    private func colorForEvent(_: String) -> Color {
        OffriiTheme.primary
    }

    private func descriptionForEvent(_ event: CircleEventResponse) -> String {
        let actor = event.actorUsername ?? NSLocalizedString("circles.detail.someone", comment: "")
        let itemName = event.targetItemName ?? ""

        switch event.eventType {
        case "item_shared":
            return String(format: NSLocalizedString("circles.event.itemShared", comment: ""), actor, itemName)
        case "item_claimed":
            // Anti-spoiler: claim events for owned items are filtered out by the backend.
            // If one reaches here, the viewer is NOT the owner — show full info.
            return String(format: NSLocalizedString("circles.event.itemClaimed", comment: ""), actor, itemName)
        case "item_unclaimed":
            return String(format: NSLocalizedString("circles.event.itemUnclaimed", comment: ""), actor, itemName)
        case "item_received":
            return String(format: NSLocalizedString("circles.event.itemReceived", comment: ""), actor, itemName)
        case "member_joined":
            let target = event.targetUsername ?? actor
            return String(format: NSLocalizedString("circles.event.memberJoined", comment: ""), target)
        case "member_left":
            let target = event.targetUsername ?? actor
            return String(format: NSLocalizedString("circles.event.memberLeft", comment: ""), target)
        case "share_rule_set":
            return String(format: NSLocalizedString("circles.event.shareRuleSet", comment: ""), actor)
        case "share_rule_removed":
            return String(format: NSLocalizedString("circles.event.shareRuleRemoved", comment: ""), actor)
        default:
            return event.eventType
        }
    }
}
