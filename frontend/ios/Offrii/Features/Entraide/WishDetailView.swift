import SwiftUI

// MARK: - WishDetailView

struct WishDetailView: View {
    let wishId: UUID
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = WishDetailViewModel()
    @State private var showReportSheet = false
    @Environment(\.dismiss) private var dismiss

    var body: some View {
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
                        // Header
                        detailHeader(wish: wish)

                        // Content
                        VStack(spacing: OffriiTheme.spacingBase) {
                            mainCard(wish: wish)
                            actionButtons(wish: wish)
                        }
                        .padding(.horizontal, OffriiTheme.spacingLG)
                        .padding(.top, -OffriiTheme.spacingLG)
                        .padding(.bottom, OffriiTheme.spacingXL)
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
        .sheet(isPresented: $showReportSheet) {
            ReportWishSheet(wishId: wishId)
                .presentationDetents([.medium])
        }
        .task {
            await viewModel.loadWish(id: wishId)
            if let wish = viewModel.wish, !wish.isMine, wish.status == .open {
                tipManager.showIfNeeded(.entraideOffer)
            }
        }
    }

    // MARK: - Header

    private func detailHeader(wish: WishDetail) -> some View {
        SectionHeader(
            title: wish.title,
            variant: .detail
        )
    }

    // MARK: - Main Card

    private func mainCard(wish: WishDetail) -> some View {
        OffriiCard {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingBase) {
                HStack {
                    WishCategoryChip(category: wish.category)
                    Spacer()
                    WishStatusBadge(status: wish.status)
                }

                Text(wish.title)
                    .font(OffriiTypography.title3)
                    .foregroundColor(OffriiTheme.text)

                if let description = wish.description, !description.isEmpty {
                    Text(description)
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.textSecondary)
                }

                cardLinksSection(wish: wish)
                cardImageSection(wish: wish)

                Divider()

                cardFooter(wish: wish)
            }
        }
    }

    @ViewBuilder
    private func cardLinksSection(wish: WishDetail) -> some View {
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
    }

    @ViewBuilder
    private func cardImageSection(wish: WishDetail) -> some View {
        if let imageUrl = wish.imageUrl, let url = URL(string: imageUrl) {
            AsyncImage(url: url) { image in
                image
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .frame(maxHeight: 200)
                    .cornerRadius(OffriiTheme.cornerRadiusSM)
                    .clipped()
            } placeholder: {
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                    .fill(OffriiTheme.surface)
                    .frame(height: 120)
                    .shimmer()
            }
        }
    }

    private func cardFooter(wish: WishDetail) -> some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            HStack {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Image(systemName: "person.fill")
                        .font(.system(size: 12))
                        .foregroundColor(OffriiTheme.textMuted)
                    Text(wish.displayName ?? NSLocalizedString("entraide.anonymous", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                }
                Spacer()
                Text(wish.createdAt, style: .relative)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.textMuted)
            }

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

    // Visitor actions
    @ViewBuilder
    private func visitorActions(wish: WishDetail) -> some View {
        if wish.status == .open {
            OffriiButton(
                NSLocalizedString("entraide.offer", comment: ""),
                variant: .primary,
                isLoading: viewModel.isActioning
            ) {
                Task { _ = await viewModel.offer(id: wish.id) }
            }
            .overlay(alignment: .top) {
                if tipManager.activeTip == .entraideOffer {
                    OffriiTooltip(
                        message: OnboardingTipManager.message(for: .entraideOffer),
                        arrow: .bottom
                    ) {
                        tipManager.dismiss(.entraideOffer)
                    }
                    .offset(y: -70)
                }
            }

            Button {
                showReportSheet = true
            } label: {
                Text("entraide.report.title")
                    .font(OffriiTypography.footnote)
                    .foregroundColor(OffriiTheme.danger)
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

    // Owner actions
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

            NavigationLink {
                WishMessagesView(wishId: wish.id, wishTitle: wish.title)
            } label: {
                HStack(spacing: OffriiTheme.spacingSM) {
                    Image(systemName: "message.fill")
                    Text("entraide.action.messages")
                }
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.primary)
                .frame(maxWidth: .infinity)
                .padding(.vertical, OffriiTheme.spacingBase)
                .background(Color.clear)
                .cornerRadius(OffriiTheme.cornerRadiusMD)
                .overlay(
                    RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                        .strokeBorder(OffriiTheme.primary, lineWidth: 1.5)
                )
            }
        } else if wish.status == .fulfilled {
            infoCard(
                icon: "gift.fill",
                text: NSLocalizedString("entraide.detail.fulfilledThanks", comment: ""),
                color: OffriiTheme.primary
            )
        }
    }

    // Donor (matched by me) actions
    @ViewBuilder
    private func donorActions(wish: WishDetail) -> some View {
        OffriiButton(
            NSLocalizedString("entraide.action.withdraw", comment: ""),
            variant: .danger,
            isLoading: viewModel.isActioning
        ) {
            Task { _ = await viewModel.withdrawOffer(id: wish.id) }
        }

        NavigationLink {
            WishMessagesView(wishId: wish.id, wishTitle: wish.title)
        } label: {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: "message.fill")
                Text("entraide.action.messages")
            }
            .font(OffriiTypography.headline)
            .foregroundColor(OffriiTheme.primary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, OffriiTheme.spacingBase)
            .background(Color.clear)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .strokeBorder(OffriiTheme.primary, lineWidth: 1.5)
            )
        }
    }

    // Info card helper
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
