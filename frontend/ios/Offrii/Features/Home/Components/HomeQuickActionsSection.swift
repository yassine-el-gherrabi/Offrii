import SwiftUI

// MARK: - HomeQuickActionsSection

struct HomeQuickActionsSection: View {
    @State private var showAddWish = false
    @State private var showCreateCircle = false
    @State private var showAddFriend = false
    @State private var showPublishNeed = false
    @State private var showWishLimitAlert = false

    private let columns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("home.quickActions.title", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)

            LazyVGrid(columns: columns, spacing: OffriiTheme.spacingSM) {
                actionTile(
                    icon: "gift.fill",
                    color: OffriiTheme.primary,
                    label: NSLocalizedString("create.addWish", comment: "")
                ) {
                    showAddWish = true
                }

                actionTile(
                    icon: "person.2.fill",
                    color: OffriiTheme.accent,
                    label: NSLocalizedString("create.createCircle", comment: "")
                ) {
                    showCreateCircle = true
                }

                actionTile(
                    icon: "person.badge.plus",
                    color: OffriiTheme.accent,
                    label: NSLocalizedString("create.addFriend", comment: "")
                ) {
                    showAddFriend = true
                }

                actionTile(
                    icon: "hand.raised.fill",
                    color: OffriiTheme.warning,
                    label: NSLocalizedString("create.publishNeed", comment: "")
                ) {
                    Task {
                        let wishes = (try? await CommunityWishService.shared.listMyWishes()) ?? []
                        let activeCount = wishes.filter {
                            $0.status == .open || $0.status == .matched || $0.status == .pending
                        }.count
                        if activeCount >= 3 {
                            showWishLimitAlert = true
                        } else {
                            showPublishNeed = true
                        }
                    }
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
        .sheet(isPresented: $showAddFriend) {
            AddFriendSheet {}
        }
        .sheet(isPresented: $showPublishNeed) {
            CreateWishSheet()
                .presentationDetents([.large])
        }
        .alert(
            NSLocalizedString("entraide.wishLimit.title", comment: ""),
            isPresented: $showWishLimitAlert
        ) {
            Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
        } message: {
            Text(NSLocalizedString("entraide.wishLimit.message", comment: ""))
        }
    }

    // MARK: - Action Tile

    private func actionTile(
        icon: String,
        color: Color,
        label: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            VStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: icon)
                    .font(.system(size: 20))
                    .foregroundColor(color)
                    .frame(width: 40, height: 40)
                    .background(color.opacity(0.12))
                    .clipShape(Circle())

                Text(label)
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(OffriiTheme.text)
                    .multilineTextAlignment(.center)
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, OffriiTheme.spacingSM)
            .padding(.horizontal, OffriiTheme.spacingXS)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
        }
        .buttonStyle(.plain)
    }
}
