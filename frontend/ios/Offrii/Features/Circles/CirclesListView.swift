import SwiftUI

struct CirclesListView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = CirclesViewModel()
    @State private var showCreateCircle = false

    var body: some View {
        ZStack {
            OffriiTheme.cardSurface.ignoresSafeArea()

            VStack(spacing: 0) {
                // Header
                ZStack {
                    OffriiTheme.primary.ignoresSafeArea(edges: .top)
                    DecorativeSquares(preset: .header)

                    HStack {
                        Text(NSLocalizedString("circles.title", comment: ""))
                            .font(OffriiTypography.largeTitle)
                            .foregroundColor(.white)

                        Spacer()

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
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.bottom, OffriiTheme.spacingLG)
                    .padding(.top, OffriiTheme.spacingXL)
                }
                .frame(minHeight: 140)

                // Action buttons
                HStack(spacing: OffriiTheme.spacingSM) {
                    Button {
                        showCreateCircle = true
                    } label: {
                        Label(NSLocalizedString("circles.create", comment: ""), systemImage: "plus.circle.fill")
                            .font(OffriiTypography.footnote)
                            .fontWeight(.semibold)
                            .foregroundColor(OffriiTheme.primary)
                            .padding(.horizontal, OffriiTheme.spacingMD)
                            .padding(.vertical, OffriiTheme.spacingSM)
                            .background(OffriiTheme.primary.opacity(0.1))
                            .cornerRadius(OffriiTheme.cornerRadiusXL)
                    }

                    NavigationLink {
                        FriendsView()
                            .environment(authManager)
                    } label: {
                        Label(NSLocalizedString("circles.shareTo", comment: ""), systemImage: "person.2.fill")
                            .font(OffriiTypography.footnote)
                            .fontWeight(.semibold)
                            .foregroundColor(OffriiTheme.primary)
                            .padding(.horizontal, OffriiTheme.spacingMD)
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
                    Spacer()
                    ProgressView()
                    Spacer()
                } else if viewModel.circles.isEmpty {
                    Spacer()
                    VStack(spacing: OffriiTheme.spacingLG) {
                        Image(systemName: "person.2.fill")
                            .font(.system(size: 48))
                            .foregroundStyle(OffriiTheme.textMuted)
                        Text(NSLocalizedString("circles.empty", comment: ""))
                            .font(OffriiTypography.body)
                            .foregroundStyle(OffriiTheme.textSecondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.horizontal, OffriiTheme.spacingXL)
                    Spacer()
                } else {
                    List {
                        ForEach(viewModel.circles) { circle in
                            NavigationLink(value: circle.id) {
                                circleRow(circle)
                            }
                            .swipeActions(edge: .trailing) {
                                Button(role: .destructive) {
                                    Task { await viewModel.deleteCircle(circle) }
                                } label: {
                                    Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
                                }
                            }
                        }
                    }
                    .listStyle(.plain)
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
        .refreshable {
            await viewModel.loadCircles()
            await viewModel.loadPendingCount()
        }
        .task {
            await viewModel.loadCircles()
            await viewModel.loadPendingCount()
        }
    }

    @ViewBuilder
    private func circleRow(_ circle: OffriiCircle) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: circle.isDirect ? "bubble.left.fill" : "person.2.fill")
                .font(.system(size: 16))
                .foregroundColor(OffriiTheme.primary)
                .frame(width: 32, height: 32)
                .background(OffriiTheme.primary.opacity(0.1))
                .clipShape(Circle())

            VStack(alignment: .leading, spacing: 2) {
                Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.text)

                if circle.isDirect {
                    Text("1-to-1")
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textMuted)
                } else {
                    Text(String(format: NSLocalizedString("circles.memberCount", comment: ""), circle.memberCount))
                        .font(OffriiTypography.caption)
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }

            Spacer()
        }
        .padding(.vertical, OffriiTheme.spacingXS)
    }
}
