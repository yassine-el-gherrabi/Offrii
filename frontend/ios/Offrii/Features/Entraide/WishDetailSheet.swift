import NukeUI
import SwiftUI

// MARK: - WishDetailSheet

struct WishDetailSheet: View {
    let wishId: UUID
    @State private var viewModel = WishDetailViewModel()
    @State private var showReportSheet = false
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                OffriiTheme.background.ignoresSafeArea()

                if viewModel.isLoading {
                    ScrollView {
                        VStack(spacing: OffriiTheme.spacingBase) {
                            SkeletonCard()
                            SkeletonCard()
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, OffriiTheme.spacingLG)
                    }
                } else if let wish = viewModel.wish {
                    ScrollView {
                        VStack(spacing: 0) {
                            // Image or gradient header
                            wishHeader(wish)

                            // Content
                            VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
                                // Title + description
                                Text(wish.title)
                                    .font(OffriiTypography.title3)
                                    .foregroundColor(OffriiTheme.text)

                                // Status badge
                                WishStatusBadge(status: wish.status)

                                // Category + author + date
                                HStack(spacing: OffriiTheme.spacingSM) {
                                    WishCategoryChip(category: wish.category)

                                    HStack(spacing: OffriiTheme.spacingXS) {
                                        Image(systemName: "person.fill")
                                            .font(.system(size: 10))
                                            .foregroundColor(OffriiTheme.textMuted)
                                        Text(wish.displayName ?? NSLocalizedString("entraide.anonymous", comment: ""))
                                            .font(OffriiTypography.caption)
                                            .foregroundColor(OffriiTheme.textMuted)
                                    }

                                    Spacer()

                                    Text(wish.createdAt, style: .relative)
                                        .font(OffriiTypography.caption)
                                        .foregroundColor(OffriiTheme.textMuted)
                                }

                                if let description = wish.description, !description.isEmpty {
                                    Text(description)
                                        .font(OffriiTypography.body)
                                        .foregroundColor(OffriiTheme.textSecondary)
                                }

                                // Links
                                if let links = wish.links, !links.isEmpty {
                                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                                        ForEach(links, id: \.self) { link in
                                            if let url = URL(string: link) {
                                                Link(destination: url) {
                                                    HStack(spacing: OffriiTheme.spacingXS) {
                                                        Image(systemName: "link")
                                                            .font(.system(size: 12))
                                                        Text(link)
                                                            .font(OffriiTypography.footnote)
                                                            .lineLimit(1)
                                                    }
                                                    .foregroundColor(OffriiTheme.primary)
                                                }
                                            }
                                        }
                                    }
                                }

                                // Matched info
                                if wish.status == .matched, let matchedName = wish.matchedWithDisplayName {
                                    HStack(spacing: OffriiTheme.spacingSM) {
                                        Image(systemName: "heart.fill")
                                            .font(.system(size: 12))
                                            .foregroundColor(OffriiTheme.accent)
                                        Text(String(
                                            format: NSLocalizedString("entraide.detail.matchedBy", comment: ""),
                                            matchedName
                                        ))
                                        .font(OffriiTypography.subheadline)
                                        .foregroundColor(OffriiTheme.accent)
                                    }
                                }

                                Divider()

                                // Action buttons
                                actionButtons(wish: wish)
                            }
                            .padding(.horizontal, OffriiTheme.spacingLG)
                            .padding(.top, OffriiTheme.spacingBase)
                            .padding(.bottom, OffriiTheme.spacingXL)

                            // Inline messages for matched wishes
                            if (wish.isMine && wish.status == .matched) || wish.isMatchedByMe {
                                Divider()
                                    .padding(.horizontal, OffriiTheme.spacingLG)

                                WishMessagesView(wishId: wish.id, wishTitle: wish.title)
                                    .frame(minHeight: 200)
                            }
                        }
                    }

                    // Success toast
                    if let success = viewModel.actionSuccess {
                        VStack {
                            Spacer()
                            Text(success)
                                .font(OffriiTypography.footnote)
                                .fontWeight(.medium)
                                .foregroundColor(.white)
                                .padding(.horizontal, OffriiTheme.spacingBase)
                                .padding(.vertical, OffriiTheme.spacingSM)
                                .background(OffriiTheme.success)
                                .cornerRadius(OffriiTheme.cornerRadiusMD)
                                .padding(.bottom, OffriiTheme.spacingXL)
                                .transition(.move(edge: .bottom).combined(with: .opacity))
                        }
                        .animation(OffriiAnimation.defaultSpring, value: viewModel.actionSuccess)
                        .onAppear {
                            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                                viewModel.actionSuccess = nil
                            }
                        }
                    }
                } else if let error = viewModel.error {
                    VStack(spacing: OffriiTheme.spacingBase) {
                        Text(error)
                            .font(OffriiTypography.body)
                            .foregroundColor(OffriiTheme.danger)
                        Button(NSLocalizedString("common.retry", comment: "")) {
                            Task { await viewModel.loadWish(id: wishId) }
                        }
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 15, weight: .semibold))
                            .foregroundColor(OffriiTheme.textSecondary)
                    }
                }

                ToolbarItem(placement: .primaryAction) {
                    Menu {
                        Button {
                            showReportSheet = true
                        } label: {
                            Label(NSLocalizedString("entraide.report.title", comment: ""), systemImage: "flag")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                            .foregroundColor(OffriiTheme.textMuted)
                    }
                }
            }
            .sheet(isPresented: $showReportSheet) {
                ReportWishSheet(wishId: wishId)
                    .presentationDetents([.medium])
            }
            .task {
                await viewModel.loadWish(id: wishId)
            }
        }
    }

    // MARK: - Header

    @ViewBuilder
    private func wishHeader(_ wish: WishDetail) -> some View {
        if let imageUrl = wish.imageUrl, let url = URL(string: imageUrl) {
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(height: 180)
                        .clipped()
                } else {
                    categoryGradientHeader(wish.category)
                }
            }
        } else {
            categoryGradientHeader(wish.category)
        }
    }

    private func categoryGradientHeader(_ category: WishCategory) -> some View {
        LinearGradient(
            colors: [category.backgroundColor, category.textColor.opacity(0.3)],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .frame(height: 140)
        .overlay(
            Image(systemName: categoryIcon(category))
                .font(.system(size: 48, weight: .light))
                .foregroundColor(.white.opacity(0.6))
        )
    }

    private func categoryIcon(_ cat: WishCategory) -> String {
        switch cat {
        case .education: return "book.fill"
        case .clothing:  return "tshirt.fill"
        case .health:    return "heart.fill"
        case .religion:  return "hands.sparkles.fill"
        case .home:      return "house.fill"
        case .children:  return "figure.and.child.holdinghands"
        case .other:     return "gift.fill"
        }
    }

    // MARK: - Action Buttons

    @ViewBuilder
    private func actionButtons(wish: WishDetail) -> some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            if wish.isMine {
                ownerActions(wish: wish)
            } else if wish.isMatchedByMe {
                donorActions(wish: wish)
            } else {
                visitorActions(wish: wish)
            }
        }
    }

    @ViewBuilder
    private func visitorActions(wish: WishDetail) -> some View {
        if wish.status == .open {
            OffriiButton(
                NSLocalizedString("entraide.offer.cta", comment: ""),
                variant: .primary,
                isLoading: viewModel.isActioning
            ) {
                Task { _ = await viewModel.offer(id: wish.id) }
            }
        } else if wish.status == .matched {
            infoCard(
                icon: "heart.fill",
                text: NSLocalizedString("entraide.detail.alreadyMatched", comment: ""),
                color: OffriiTheme.accent
            )
        } else if wish.status == .fulfilled {
            infoCard(
                icon: "gift.fill",
                text: NSLocalizedString("entraide.detail.fulfilled", comment: ""),
                color: OffriiTheme.primary
            )
        }
    }

    @ViewBuilder
    private func ownerActions(wish: WishDetail) -> some View {
        if wish.status == .open {
            OffriiButton(
                NSLocalizedString("entraide.action.close", comment: ""),
                variant: .secondary,
                isLoading: viewModel.isActioning
            ) {
                Task { _ = await viewModel.closeWish(id: wish.id) }
            }
        } else if wish.status == .matched {
            OffriiButton(
                NSLocalizedString("entraide.action.confirm", comment: ""),
                variant: .primary,
                isLoading: viewModel.isActioning
            ) {
                Task { _ = await viewModel.confirm(id: wish.id) }
            }

            OffriiButton(
                NSLocalizedString("entraide.action.reject", comment: ""),
                variant: .danger,
                isLoading: viewModel.isActioning
            ) {
                Task { _ = await viewModel.rejectOffer(id: wish.id) }
            }
        } else if wish.status == .fulfilled {
            infoCard(
                icon: "gift.fill",
                text: NSLocalizedString("entraide.detail.fulfilledThanks", comment: ""),
                color: OffriiTheme.primary
            )
        }
    }

    @ViewBuilder
    private func donorActions(wish: WishDetail) -> some View {
        OffriiButton(
            NSLocalizedString("entraide.action.withdraw", comment: ""),
            variant: .danger,
            isLoading: viewModel.isActioning
        ) {
            Task { _ = await viewModel.withdrawOffer(id: wish.id) }
        }
    }

    private func infoCard(icon: String, text: String, color: Color) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: icon)
                .foregroundColor(color)
            Text(text)
                .font(OffriiTypography.subheadline)
                .foregroundColor(color)
        }
        .frame(maxWidth: .infinity)
        .padding(OffriiTheme.spacingBase)
        .background(color.opacity(0.1))
        .cornerRadius(OffriiTheme.cornerRadiusMD)
    }
}
