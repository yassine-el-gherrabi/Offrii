import SwiftUI

// MARK: - CircleListContent

struct CircleListContent: View {
    var viewModel: CirclesViewModel
    @Binding var showCreateCircle: Bool
    @Environment(OnboardingTipManager.self) private var tipManager

    var body: some View {
        if viewModel.isLoadingCircles && viewModel.circles.isEmpty {
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingSM) {
                    ForEach(0..<5, id: \.self) { _ in
                        SkeletonRow(height: 86)
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
                    // Search bar
                    searchBar
                        .padding(.horizontal, OffriiTheme.spacingXS)

                    createCircleButton

                    ForEach(viewModel.filteredCircles) { circle in
                        NavigationLink(value: circle.id) {
                            CircleCardRow(circle: circle)
                        }
                        .buttonStyle(.plain)
                        .contextMenu {
                            Button {
                                // Edit handled by detail view
                            } label: {
                                Label(
                                    NSLocalizedString("circles.context.edit", comment: ""),
                                    systemImage: "pencil"
                                )
                            }

                            Divider()

                            Button(role: .destructive) {
                                Task { await viewModel.deleteCircle(circle) }
                            } label: {
                                Label(
                                    NSLocalizedString("circles.context.delete", comment: ""),
                                    systemImage: "trash"
                                )
                            }
                        }
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.vertical, OffriiTheme.spacingSM)
            }
        }
    }

    // MARK: - Create Button

    @ViewBuilder
    private var createCircleButton: some View {
        Button {
            showCreateCircle = true
        } label: {
            Label(
                NSLocalizedString("circles.create", comment: ""),
                systemImage: "plus.circle.fill"
            )
            .font(OffriiTypography.footnote)
            .fontWeight(.semibold)
            .foregroundColor(OffriiTheme.primary)
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
            .background(OffriiTheme.primary.opacity(0.1))
            .cornerRadius(OffriiTheme.cornerRadiusXL)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
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
    }

    // MARK: - Search Bar

    @ViewBuilder
    private var searchBar: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: "magnifyingglass")
                .foregroundColor(OffriiTheme.textMuted)

            TextField(
                NSLocalizedString("circles.search.placeholder", comment: ""),
                text: Bindable(viewModel).circleSearchQuery
            )
            .font(OffriiTypography.body)
            .autocapitalization(.none)
            .autocorrectionDisabled()

            if !viewModel.circleSearchQuery.isEmpty {
                Button {
                    viewModel.circleSearchQuery = ""
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }
        }
        .padding(OffriiTheme.spacingSM)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusSM)
        .overlay(
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                .stroke(OffriiTheme.border, lineWidth: 1)
        )
    }
}
