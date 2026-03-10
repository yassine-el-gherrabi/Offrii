import SwiftUI

// MARK: - MyWishesView

struct MyWishesView: View {
    @State private var viewModel = MyWishesViewModel()

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                HeaderView(
                    title: NSLocalizedString("entraide.myWishes.title", comment: ""),
                    subtitle: nil
                )

                // Content
                if viewModel.isLoading {
                    Spacer()
                    ProgressView()
                    Spacer()
                } else if viewModel.wishes.isEmpty {
                    Spacer()
                    emptyState
                    Spacer()
                } else {
                    wishList
                }
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .task {
            await viewModel.loadMyWishes()
        }
    }

    // MARK: - Wish List

    private var wishList: some View {
        ScrollView {
            LazyVStack(spacing: OffriiTheme.spacingMD) {
                ForEach(viewModel.wishes) { wish in
                    NavigationLink {
                        WishDetailView(wishId: wish.id)
                    } label: {
                        myWishCard(wish: wish)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
        .refreshable {
            await viewModel.loadMyWishes()
        }
    }

    // MARK: - My Wish Card

    private func myWishCard(wish: MyWish) -> some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            HStack {
                WishCategoryChip(category: wish.category)
                Spacer()
                WishStatusBadge(status: wish.status)
            }

            Text(wish.title)
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .lineLimit(2)
                .multilineTextAlignment(.leading)

            if let description = wish.description, !description.isEmpty {
                Text(description)
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textSecondary)
                    .lineLimit(2)
                    .multilineTextAlignment(.leading)
            }

            myWishCardMeta(wish: wish)

            HStack {
                Spacer()
                Text(wish.createdAt, style: .relative)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }
        }
        .padding(OffriiTheme.spacingMD)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(
            color: OffriiTheme.cardShadowColor,
            radius: OffriiTheme.cardShadowRadius,
            x: 0,
            y: OffriiTheme.cardShadowY
        )
    }

    @ViewBuilder
    private func myWishCardMeta(wish: MyWish) -> some View {
        if let matchedName = wish.matchedWithDisplayName {
            HStack(spacing: OffriiTheme.spacingXS) {
                Image(systemName: "heart.fill")
                    .font(.system(size: 10))
                    .foregroundColor(OffriiTheme.accent)
                Text(String(
                    format: NSLocalizedString("entraide.detail.matchedBy", comment: ""),
                    matchedName
                ))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.accent)
            }
        }

        if wish.reportCount > 0 {
            HStack(spacing: OffriiTheme.spacingXS) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .font(.system(size: 10))
                    .foregroundColor(OffriiTheme.danger)
                Text(String(
                    format: NSLocalizedString("entraide.myWishes.reports", comment: ""),
                    wish.reportCount
                ))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.danger)
            }
        }

        if let note = wish.moderationNote, !note.isEmpty {
            HStack(spacing: OffriiTheme.spacingXS) {
                Image(systemName: "info.circle.fill")
                    .font(.system(size: 10))
                    .foregroundColor(OffriiTheme.accent)
                Text(note)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.accent)
            }
        }
    }

    // MARK: - Empty State

    private var emptyState: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            Image(systemName: "tray")
                .font(.system(size: 48))
                .foregroundColor(OffriiTheme.textMuted)

            Text("entraide.myWishes.empty")
                .font(OffriiTypography.title3)
                .foregroundColor(OffriiTheme.text)

            Text("entraide.myWishes.emptySubtitle")
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, OffriiTheme.spacingXL)
        }
    }
}
