import SwiftUI

// MARK: - CreateWishSheet

struct CreateWishSheet: View {
    @State private var viewModel = CreateWishViewModel()
    @Environment(\.dismiss) private var dismiss
    var onCreated: (() -> Void)?

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                    // Title field
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text("entraide.create.titleLabel")
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        OffriiTextField(
                            label: "",
                            text: $viewModel.title,
                            placeholder: NSLocalizedString("entraide.create.titlePlaceholder", comment: "")
                        )
                    }

                    // Description field
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text("entraide.create.descriptionLabel")
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        TextEditor(text: $viewModel.description)
                            .font(OffriiTypography.body)
                            .frame(minHeight: 100)
                            .padding(OffriiTheme.spacingSM)
                            .background(OffriiTheme.surface)
                            .cornerRadius(OffriiTheme.cornerRadiusSM)
                            .overlay(
                                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                                    .strokeBorder(
                                        viewModel.isDescriptionOverLimit
                                            ? OffriiTheme.danger
                                            : OffriiTheme.border,
                                        lineWidth: 1
                                    )
                            )

                        HStack {
                            Spacer()
                            Text("\(viewModel.descriptionCount)/2000")
                                .font(OffriiTypography.caption)
                                .foregroundColor(
                                    viewModel.isDescriptionOverLimit
                                        ? OffriiTheme.danger
                                        : OffriiTheme.textMuted
                                )
                        }
                    }

                    // Category picker
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        Text("entraide.create.categoryLabel")
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        CategoryGridPicker(selectedCategory: $viewModel.selectedCategory)
                    }

                    // Anonymous toggle
                    Toggle(isOn: $viewModel.isAnonymous) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("entraide.create.anonymous")
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.text)
                            Text("entraide.create.anonymousHint")
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                    }
                    .tint(OffriiTheme.primary)

                    // Image URL
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                        Text("entraide.create.imageUrlLabel")
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        OffriiTextField(
                            label: "",
                            text: $viewModel.imageUrl,
                            placeholder: "https://...",
                            keyboardType: .URL,
                            autocapitalization: .never
                        )
                    }

                    // Links
                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        Text("entraide.create.linksLabel")
                            .font(OffriiTypography.headline)
                            .foregroundColor(OffriiTheme.text)

                        ForEach(viewModel.links.indices, id: \.self) { index in
                            HStack {
                                OffriiTextField(
                                    label: "",
                                    text: $viewModel.links[index],
                                    placeholder: "https://...",
                                    keyboardType: .URL,
                                    autocapitalization: .never
                                )

                                if viewModel.links.count > 1 {
                                    Button {
                                        viewModel.removeLink(at: index)
                                    } label: {
                                        Image(systemName: "xmark.circle.fill")
                                            .foregroundColor(OffriiTheme.textMuted)
                                    }
                                }
                            }
                        }

                        if viewModel.links.count < 10 {
                            Button {
                                viewModel.addLink()
                            } label: {
                                HStack(spacing: OffriiTheme.spacingXS) {
                                    Image(systemName: "plus.circle")
                                    Text("entraide.create.addLink")
                                }
                                .font(OffriiTypography.footnote)
                                .foregroundColor(OffriiTheme.primary)
                            }
                        }
                    }

                    // Error
                    if let error = viewModel.error {
                        Text(error)
                            .font(OffriiTypography.footnote)
                            .foregroundColor(OffriiTheme.danger)
                    }

                    // Submit button
                    OffriiButton(
                        NSLocalizedString("entraide.create.submit", comment: ""),
                        variant: .primary,
                        isLoading: viewModel.isSubmitting,
                        isDisabled: !viewModel.isFormValid
                    ) {
                        Task {
                            if await viewModel.submit() {
                                onCreated?()
                                dismiss()
                            }
                        }
                    }
                }
                .padding(OffriiTheme.spacingLG)
            }
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("entraide.create.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(NSLocalizedString("common.cancel", comment: "")) {
                        dismiss()
                    }
                }
            }
        }
    }
}
