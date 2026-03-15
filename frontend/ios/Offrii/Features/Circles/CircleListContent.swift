import SwiftUI

// MARK: - CircleListContent

struct CircleListContent: View {
    var viewModel: CirclesViewModel

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
                subtitle: NSLocalizedString("circles.emptySubtitle", comment: "")
            )
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: OffriiTheme.spacingSM) {
                    // Search bar
                    searchBar
                        .padding(.horizontal, OffriiTheme.spacingXS)

                    ForEach(viewModel.filteredCircles) { circle in
                        NavigationLink(value: circle.id) {
                            CircleCardRow(circle: circle)
                        }
                        .buttonStyle(.plain)
                        .contextMenu {
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
