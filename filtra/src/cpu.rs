use filtra_kernels::{ColorSpace, IntRect, Pixmap, PorterDuff, Source, Target};
use filtra_plan::{CompositeOperator, Plan, Step};

pub(crate) fn execute(plan: &Plan, source: &Pixmap) -> Pixmap {
    let assignment = filtra_plan::assign_buffers(&plan.steps, &plan.rects, plan.output);
    let images: Vec<Image> = (plan.rects.iter().zip(assignment.images))
        .map(|(&rect, buffer)| Image { buffer, rect })
        .collect();
    let mut buffers: Vec<Pixmap> = (0..assignment.count).map(|_| Pixmap::new(0, 0)).collect();

    for step in &plan.steps {
        execute_step(step, &images, source, &mut buffers);
    }

    let mut output = Pixmap::new(source.width(), source.height());
    let result = images[plan.output.0];
    filtra_kernels::offset(read(&buffers, result), output.target(0, 0), 0, 0);

    output
}

#[derive(Clone, Copy)]
struct Image {
    buffer: usize,
    rect: filtra_plan::IntRect,
}

fn execute_step(step: &Step, images: &[Image], source: &Pixmap, buffers: &mut [Pixmap]) {
    let image = |id: filtra_plan::ImageId| images[id.0];

    match step {
        Step::SourceGraphic { dst } => write(buffers, image(*dst), |_, target| {
            filtra_kernels::offset(source.source(0, 0), target, 0, 0);
        }),
        Step::SourceAlpha { dst } => write(buffers, image(*dst), |_, target| {
            filtra_kernels::alpha(source.source(0, 0), target);
        }),
        Step::Convert { src, dst, from, to } => write(buffers, image(*dst), |buffers, target| {
            let (from, to) = (color_space(*from), color_space(*to));
            filtra_kernels::convert(read(buffers, image(*src)), target, from, to);
        }),
        Step::Flood { dst, color } => write(buffers, image(*dst), |_, target| {
            filtra_kernels::flood(target, *color);
        }),
        Step::Offset { src, dst, dx, dy } => write(buffers, image(*dst), |buffers, target| {
            filtra_kernels::offset(read(buffers, image(*src)), target, *dx, *dy);
        }),
        Step::Tile { src, cell, dst } => write(buffers, image(*dst), |buffers, target| {
            let cell = IntRect::new(cell.x0, cell.y0, cell.x1, cell.y1);
            filtra_kernels::tile(read(buffers, image(*src)), cell, target);
        }),
        Step::Composite {
            src,
            backdrop,
            operator,
            dst,
        } => write(buffers, image(*dst), |buffers, target| {
            let src = read(buffers, image(*src));
            let backdrop = read(buffers, image(*backdrop));
            composite(src, backdrop, *operator, target);
        }),
        Step::Merge { srcs, dst } => write(buffers, image(*dst), |buffers, target| {
            let srcs: Vec<Source<'_>> = srcs.iter().map(|src| read(buffers, image(*src))).collect();
            filtra_kernels::merge(&srcs, target);
        }),
        Step::ColorMatrix { src, dst, matrix } => write(buffers, image(*dst), |buffers, target| {
            filtra_kernels::color_matrix(read(buffers, image(*src)), target, matrix);
        }),
        Step::ComponentTransfer {
            src,
            dst,
            functions,
        } => write(buffers, image(*dst), |buffers, target| {
            let functions = functions.each_ref().map(Vec::as_slice);
            filtra_kernels::component_transfer(read(buffers, image(*src)), target, functions);
        }),
    }
}

fn write(buffers: &mut [Pixmap], dst: Image, f: impl FnOnce(&[Pixmap], Target<'_>)) {
    let (width, height) = (dst.rect.width(), dst.rect.height());
    let mut data = std::mem::replace(&mut buffers[dst.buffer], Pixmap::new(0, 0)).into_data();
    data.resize(width as usize * height as usize * 4, 0);

    let mut pixmap = Pixmap::from_data(data, width, height);
    f(buffers, pixmap.target(dst.rect.x0, dst.rect.y0));
    buffers[dst.buffer] = pixmap;
}

fn read(buffers: &[Pixmap], image: Image) -> Source<'_> {
    buffers[image.buffer].source(image.rect.x0, image.rect.y0)
}

fn color_space(space: filtra_plan::ColorSpace) -> ColorSpace {
    match space {
        filtra_plan::ColorSpace::Srgb => ColorSpace::Srgb,
        filtra_plan::ColorSpace::LinearRgb => ColorSpace::LinearRgb,
    }
}

fn composite(src: Source<'_>, backdrop: Source<'_>, operator: CompositeOperator, dst: Target<'_>) {
    let operator = match operator {
        CompositeOperator::Over => PorterDuff::Over,
        CompositeOperator::In => PorterDuff::In,
        CompositeOperator::Out => PorterDuff::Out,
        CompositeOperator::Atop => PorterDuff::Atop,
        CompositeOperator::Xor => PorterDuff::Xor,
        CompositeOperator::Lighter => PorterDuff::Lighter,
        CompositeOperator::Arithmetic { k1, k2, k3, k4 } => {
            return filtra_kernels::arithmetic(src, backdrop, [k1, k2, k3, k4], dst);
        }
    };

    filtra_kernels::composite(src, backdrop, operator, dst);
}
