import SwiftUI

// MARK: - QuickStartSection

struct QuickStartSection: View {
    let completedActions: Set<QuickStartAction>

    @State private var showAddWish = false
    @State private var showCreateCircle = false

    private struct ActionCard: Identifiable {
        let id = UUID()
        let action: QuickStartAction
        let icon: String
        let label: String
    }

    private var actionCards: [ActionCard] {
        [
            ActionCard(
                action: .addWish,
                icon: "gift.fill",
                label: NSLocalizedString("home.quickStart.addWish", comment: "")
            ),
            ActionCard(
                action: .createCircle,
                icon: "person.2.fill",
                label: NSLocalizedString("home.quickStart.createCircle", comment: "")
            ),
            ActionCard(
                action: .inviteFriend,
                icon: "person.badge.plus",
                label: NSLocalizedString("home.quickStart.inviteFriend", comment: "")
            ),
            ActionCard(
                action: .shareList,
                icon: "square.and.arrow.up",
                label: NSLocalizedString("home.quickStart.shareList", comment: "")
            ),
        ]
    }

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("home.quickStart.title", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)

            LazyVGrid(
                columns: [
                    GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
                    GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
                ],
                spacing: OffriiTheme.spacingSM
            ) {
                ForEach(actionCards) { card in
                    quickStartCard(card)
                }
            }
        }
        .sheet(isPresented: $showAddWish) {
            QuickAddSheet { name, price, categoryId, priority, imageUrl, links, isPrivate in
                _ = try? await ItemService.shared.createItem(
                    name: name,
                    estimatedPrice: price,
                    priority: priority,
                    categoryId: categoryId,
                    imageUrl: imageUrl,
                    links: links,
                    isPrivate: isPrivate
                )
                return true
            }
        }
        .sheet(isPresented: $showCreateCircle) {
            CreateCircleSheet { _ in }
                .presentationDetents([.medium])
        }
    }

    // MARK: - Quick Start Card

    private func quickStartCard(_ card: ActionCard) -> some View {
        let isDone = completedActions.contains(card.action)

        return Button {
            handleAction(card.action)
        } label: {
            VStack(spacing: OffriiTheme.spacingSM) {
                ZStack {
                    Circle()
                        .fill(isDone ? OffriiTheme.success.opacity(0.15) : OffriiTheme.primary.opacity(0.1))
                        .frame(width: 44, height: 44)

                    Image(systemName: isDone ? "checkmark" : card.icon)
                        .font(.system(size: 18))
                        .foregroundColor(isDone ? OffriiTheme.success : OffriiTheme.primary)
                }

                Text(isDone ? NSLocalizedString("home.quickStart.completed", comment: "") : card.label)
                    .font(OffriiTypography.caption)
                    .foregroundColor(isDone ? OffriiTheme.success : OffriiTheme.text)
                    .multilineTextAlignment(.center)
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, OffriiTheme.spacingBase)
            .padding(.horizontal, OffriiTheme.spacingSM)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
            .opacity(isDone ? 0.7 : 1.0)
        }
        .buttonStyle(.plain)
        .disabled(isDone)
    }

    private func handleAction(_ action: QuickStartAction) {
        switch action {
        case .addWish:
            showAddWish = true
        case .createCircle:
            showCreateCircle = true
        case .inviteFriend, .shareList:
            // These navigate — handled by parent context
            break
        }
    }
}
