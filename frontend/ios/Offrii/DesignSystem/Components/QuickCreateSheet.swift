import SwiftUI

// MARK: - QuickCreateSheet

struct QuickCreateSheet: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @State private var navigateToAddWish = false
    @State private var navigateToCreateCircle = false
    @State private var navigateToAddFriend = false
    @State private var navigateToPublishNeed = false
    @State private var showWishLimitAlert = false
    @State private var showEligibilityAlert = false

    var body: some View {
        NavigationStack {
            VStack(spacing: OffriiTheme.spacingBase) {
                createOption(
                    icon: "gift.fill",
                    iconColor: OffriiTheme.primary,
                    title: NSLocalizedString("create.addWish", comment: ""),
                    subtitle: NSLocalizedString("create.addWishSubtitle", comment: "")
                ) {
                    navigateToAddWish = true
                }

                createOption(
                    icon: "person.2.fill",
                    iconColor: OffriiTheme.accent,
                    title: NSLocalizedString("create.createCircle", comment: ""),
                    subtitle: NSLocalizedString("create.createCircleSubtitle", comment: "")
                ) {
                    navigateToCreateCircle = true
                }

                createOption(
                    icon: "person.badge.plus",
                    iconColor: OffriiTheme.accent,
                    title: NSLocalizedString("create.addFriend", comment: ""),
                    subtitle: NSLocalizedString("create.addFriendSubtitle", comment: "")
                ) {
                    navigateToAddFriend = true
                }

                createOption(
                    icon: "hand.raised.fill",
                    iconColor: OffriiTheme.warning,
                    title: NSLocalizedString("create.publishNeed", comment: ""),
                    subtitle: NSLocalizedString("create.publishNeedSubtitle", comment: ""),
                    isDisabled: !EntraideEligibility(user: authManager.currentUser).isEligible
                ) {
                    guard EntraideEligibility(user: authManager.currentUser).isEligible else {
                        showEligibilityAlert = true
                        return
                    }
                    Task {
                        let wishes = (try? await CommunityWishService.shared.listMyWishes()) ?? []
                        let activeCount = wishes.filter {
                            $0.status == .open || $0.status == .matched || $0.status == .pending
                        }.count
                        if activeCount >= 3 {
                            showWishLimitAlert = true
                        } else {
                            navigateToPublishNeed = true
                        }
                    }
                }

                Spacer()
            }
            .padding(.top, OffriiTheme.spacingBase)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .navigationTitle(NSLocalizedString("create.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundColor(OffriiTheme.textSecondary)
                    }
                }
            }
        }
        .sheet(isPresented: $navigateToAddWish) {
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
        .sheet(isPresented: $navigateToCreateCircle) {
            CreateCircleSheet { _ in }
                .presentationDetents([.medium])
        }
        .sheet(isPresented: $navigateToAddFriend) {
            AddFriendSheet {}
        }
        .sheet(isPresented: $navigateToPublishNeed) {
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
        .alert(
            NSLocalizedString("entraide.eligibility.title", comment: ""),
            isPresented: $showEligibilityAlert
        ) {
            Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
        } message: {
            Text(NSLocalizedString("entraide.eligibility.message", comment: ""))
        }
    }

    // MARK: - Option Card

    private func createOption(
        icon: String,
        iconColor: Color,
        title: String,
        subtitle: String,
        isDisabled: Bool = false,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: OffriiTheme.spacingBase) {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .fill(iconColor.opacity(0.12))
                    .frame(width: 48, height: 48)
                    .overlay(
                        Image(systemName: icon)
                            .font(.system(size: 20))
                            .foregroundColor(iconColor)
                    )

                VStack(alignment: .leading, spacing: OffriiTheme.spacingXXS) {
                    Text(title)
                        .font(OffriiTypography.headline)
                        .foregroundColor(OffriiTheme.text)

                    Text(subtitle)
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }

                Spacer()

                Image(systemName: "chevron.right")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundColor(OffriiTheme.textMuted)
            }
            .padding(OffriiTheme.spacingBase)
            .background(OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .shadow(
                color: OffriiTheme.cardShadowColor,
                radius: OffriiTheme.cardShadowRadius,
                x: 0,
                y: OffriiTheme.cardShadowY
            )
            .opacity(isDisabled ? 0.45 : 1.0)
        }
        .buttonStyle(.plain)
    }
}
