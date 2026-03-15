import SwiftUI

struct CirclesListView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = CirclesViewModel()
    @State private var showCreateCircle = false

    var body: some View {
        ZStack {
            OffriiTheme.background.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                SectionHeader(
                    title: NSLocalizedString("circles.title", comment: ""),
                    variant: .cercles
                ) {
                    HStack(spacing: 12) {
                        NavigationLink {
                            PendingRequestsView()
                                .environment(authManager)
                        } label: {
                            ZStack(alignment: .topTrailing) {
                                Image(systemName: "bell.fill")
                                    .font(.system(size: 20))
                                    .foregroundColor(.white)

                                if viewModel.pendingRequestsCount > 0 {
                                    Text("\(viewModel.pendingRequestsCount)")
                                        .font(.system(size: 10, weight: .bold))
                                        .foregroundColor(.white)
                                        .padding(4)
                                        .background(OffriiTheme.danger)
                                        .clipShape(Circle())
                                        .offset(x: 8, y: -8)
                                }
                            }
                        }

                        NavigationLink(destination: ProfileView()) {
                            ProfileAvatarButton(
                                initials: ProfileAvatarButton.initials(from: authManager.currentUser?.displayName)
                            )
                        }
                    }
                }

                // Action buttons
                HStack(spacing: OffriiTheme.spacingSM) {
                    Button {
                        showCreateCircle = true
                    } label: {
                        Label(NSLocalizedString("circles.create", comment: ""), systemImage: "plus.circle.fill")
                            .font(OffriiTypography.footnote)
                            .fontWeight(.semibold)
                            .foregroundColor(OffriiTheme.primary)
                            .padding(.horizontal, OffriiTheme.spacingBase)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(OffriiTheme.primary.opacity(0.1))
                            .cornerRadius(OffriiTheme.cornerRadiusXL)
                    }
                    .overlay(alignment: .bottom) {
                        if tipManager.activeTip == .circlesCreate {
                            OffriiTooltip(
                                message: OnboardingTipManager.message(for: .circlesCreate),
                                arrow: .top
                            ) {
                                tipManager.dismiss(.circlesCreate)
                            }
                            .offset(y: 60)
                        }
                    }

                    NavigationLink {
                        FriendsView()
                            .environment(authManager)
                    } label: {
                        Label(NSLocalizedString("circles.shareTo", comment: ""), systemImage: "person.2.fill")
                            .font(OffriiTypography.footnote)
                            .fontWeight(.semibold)
                            .foregroundColor(OffriiTheme.primary)
                            .padding(.horizontal, OffriiTheme.spacingBase)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(OffriiTheme.primary.opacity(0.1))
                            .cornerRadius(OffriiTheme.cornerRadiusXL)
                    }

                    Spacer()
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)

                // Content
                if viewModel.isLoading && viewModel.circles.isEmpty {
                    ScrollView {
                        LazyVStack(spacing: OffriiTheme.spacingSM) {
                            ForEach(0..<5, id: \.self) { _ in
                                SkeletonRow(height: 76)
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingBase)
                        .padding(.top, OffriiTheme.spacingBase)
                    }
                } else if viewModel.circles.isEmpty {
                    Spacer()
                    OffriiEmptyState(
                        icon: "person.2.fill",
                        title: NSLocalizedString("circles.empty", comment: ""),
                        subtitle: NSLocalizedString("circles.emptySubtitle", comment: ""),
                        ctaTitle: NSLocalizedString("circles.create", comment: ""),
                        ctaAction: { showCreateCircle = true }
                    )
                    Spacer()
                } else {
                    ScrollView {
                        LazyVStack(spacing: OffriiTheme.spacingSM) {
                            ForEach(viewModel.circles) { circle in
                                NavigationLink(value: circle.id) {
                                    CircleCardRow(circle: circle)
                                }
                                .buttonStyle(.plain)
                                .contextMenu {
                                    Button(role: .destructive) {
                                        Task { await viewModel.deleteCircle(circle) }
                                    } label: {
                                        Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
                                    }
                                }
                            }
                        }
                        .padding(.horizontal, OffriiTheme.spacingBase)
                        .padding(.vertical, OffriiTheme.spacingSM)
                    }
                    .refreshable {
                        await viewModel.loadCircles()
                        await viewModel.loadPendingCount()
                    }
                }
            }
        }
        .navigationDestination(for: UUID.self) { circleId in
            CircleDetailView(circleId: circleId)
                .environment(authManager)
        }
        .sheet(isPresented: $showCreateCircle) {
            CreateCircleSheet { _ in
                Task { await viewModel.loadCircles() }
            }
        }
        .task {
            await viewModel.loadCircles()
            await viewModel.loadPendingCount()
            tipManager.showIfNeeded(.circlesCreate)
        }
    }
}
