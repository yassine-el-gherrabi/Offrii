import PhotosUI
import SwiftUI

// MARK: - OffriiImagePicker

struct OffriiImagePicker: View {
    @Binding var selectedImage: UIImage?
    var existingImageUrl: URL?
    @State private var pickerItem: PhotosPickerItem?

    private var hasImage: Bool {
        selectedImage != nil || existingImageUrl != nil
    }

    var body: some View {
        ZStack {
            // Image or placeholder
            if let selectedImage {
                Image(uiImage: selectedImage)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .frame(maxWidth: .infinity)
                    .frame(height: 180)
                    .clipped()
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
            } else if let existingImageUrl {
                AsyncImage(url: existingImageUrl) { phase in
                    switch phase {
                    case .success(let image):
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(maxWidth: .infinity)
                            .frame(height: 180)
                            .clipped()
                            .cornerRadius(OffriiTheme.cornerRadiusMD)
                    default:
                        placeholderView
                    }
                }
            } else {
                placeholderView
            }

            // Overlay controls
            if hasImage {
                // Change / Remove overlay
                VStack {
                    HStack {
                        Spacer()
                        // Remove button
                        Button {
                            withAnimation(OffriiAnimation.snappy) {
                                selectedImage = nil
                                pickerItem = nil
                            }
                        } label: {
                            Image(systemName: "xmark")
                                .font(.system(size: 12, weight: .bold))
                                .foregroundColor(.white)
                                .frame(width: 28, height: 28)
                                .background(.black.opacity(0.4))
                                .clipShape(Circle())
                                .overlay(Circle().strokeBorder(.white.opacity(0.2), lineWidth: 0.5))
                        }
                    }
                    .padding(OffriiTheme.spacingSM)

                    Spacer()

                    // Change photo button
                    PhotosPicker(selection: $pickerItem, matching: .images) {
                        HStack(spacing: 6) {
                            Image(systemName: "camera.fill")
                                .font(.system(size: 12))
                            Text(NSLocalizedString("imagePicker.change", comment: ""))
                                .font(.system(size: 12, weight: .medium))
                        }
                        .foregroundColor(.white)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(.black.opacity(0.4))
                        .clipShape(Capsule())
                        .overlay(Capsule().strokeBorder(.white.opacity(0.2), lineWidth: 0.5))
                    }
                    .padding(.bottom, OffriiTheme.spacingSM)
                }
            }
        }
        .frame(maxWidth: .infinity)
        .frame(height: hasImage ? 180 : nil)
        .onChange(of: pickerItem) { _, newItem in
            Task {
                if let data = try? await newItem?.loadTransferable(type: Data.self),
                   let uiImage = UIImage(data: data) {
                    withAnimation(OffriiAnimation.defaultSpring) {
                        selectedImage = uiImage
                    }
                }
            }
        }
    }

    // Placeholder — only shown when no image
    private var placeholderView: some View {
        PhotosPicker(selection: $pickerItem, matching: .images) {
            VStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: "camera.fill")
                    .font(.system(size: 24))
                    .foregroundColor(OffriiTheme.textMuted)

                Text(NSLocalizedString("imagePicker.add", comment: ""))
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(OffriiTheme.textMuted)
            }
            .frame(maxWidth: .infinity)
            .frame(height: 100)
            .background(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .strokeBorder(OffriiTheme.border, style: StrokeStyle(lineWidth: 1.5, dash: [6]))
            )
        }
    }
}
